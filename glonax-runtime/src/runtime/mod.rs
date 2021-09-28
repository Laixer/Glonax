use glonax_core::{
    motion::Motion,
    operand::{Context, Operand},
};

use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    device::{DeviceDescriptor, DeviceManager, InputDevice, MetricDevice, MotionDevice},
    Config,
};

mod error;
pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

mod builder;
pub use self::builder::Builder;

#[derive(Debug)]
pub enum RuntimeEvent {
    /// Request to drive motion.
    DriveMotion(Motion),
    /// Request to shutdown runtime core.
    Shutdown,
}

unsafe impl Sync for RuntimeEvent {}
unsafe impl Send for RuntimeEvent {}

pub struct Dispatch(Sender<RuntimeEvent>);

impl Dispatch {
    // FUTURE: Maybe rename in the future
    /// Request motion.
    ///
    /// Post motion request on the runtime queue. This method will
    /// *not* wait until the action is executed.
    #[inline]
    async fn motion(
        &self,
        motion: Motion,
    ) -> std::result::Result<(), tokio::sync::mpsc::error::SendError<RuntimeEvent>> {
        self.0.send(RuntimeEvent::DriveMotion(motion)).await
    }

    /// Request runtime shutdown.
    ///
    /// This is the recommended way to shutdown the runtime. Some
    /// subsystems may need time to close resources or dispose
    /// managed objects.
    ///
    /// This method will *not* wait until the action is executed.
    #[inline]
    pub async fn gracefull_shutdown(
        &self,
    ) -> std::result::Result<(), tokio::sync::mpsc::error::SendError<RuntimeEvent>> {
        self.0.send(RuntimeEvent::Shutdown).await
    }
}

pub(super) struct RuntimeSettings {
    timer_interval: u64,
}

impl From<&Config> for RuntimeSettings {
    fn from(_config: &Config) -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self { timer_interval: 15 }
    }
}

pub struct Runtime<A, K> {
    /// Runtime operand.
    pub(super) operand: K,
    /// The standard motion device.
    pub(super) motion_device: DeviceDescriptor<A>,
    /// The standard motion device.
    pub(super) metric_devices: Vec<DeviceDescriptor<dyn MetricDevice + Send>>,
    /// Runtime event bus.
    pub(super) event_bus: (Sender<RuntimeEvent>, Receiver<RuntimeEvent>),
    /// Program queue.
    pub(super) program_queue: (Sender<i32>, Option<Receiver<i32>>),
    /// Runtime settings.
    pub(super) settings: RuntimeSettings,
    /// Task pool.
    pub(super) task_pool: Vec<JoinHandle<()>>,
    /// Device manager.
    pub(super) device_manager: DeviceManager,
}

impl<A, K> Runtime<A, K> {
    /// Create a runtime dispatcher.
    ///
    /// The dispatcher is the input channel to the runtime event queue. This
    /// is the recommended method to post to the event queue.
    #[inline]
    pub fn dispatch(&self) -> Dispatch {
        Dispatch(self.event_bus.0.clone())
    }

    /// Spawn background task.
    ///
    /// Run a future in the background. The background task is supposed to run
    /// for a substantial amount of time, or as long as the runtime itself. The
    /// task handle is stored and called with an abort instruction when the
    /// runtime exits its loop.
    #[inline]
    fn spawn<T>(&mut self, future: T)
    where
        T: std::future::Future<Output = ()> + Send,
        T: 'static,
    {
        self.task_pool.push(tokio::task::spawn(future));
    }

    /// Run idle time operations.
    ///
    /// This method is called when the runtime is idle. Operations run here may
    /// *never* block, halt or otherwise obstruct the runtime. Doing so will
    /// sarve the runtime and can increase the event latency.
    async fn idle_tlime(&mut self) {
        self.device_manager.idle_time().await;
    }
}

impl<A: MotionDevice, K> Runtime<A, K> {
    /// Start the runtime.
    ///
    /// The runtime will process the events from the event bus. The runtime
    /// should only every break out of the loop if shutdown was requested.
    pub(super) async fn run(&mut self) {
        use tokio::time::{sleep, Duration};

        loop {
            let wait = sleep(Duration::from_secs(self.settings.timer_interval));

            tokio::select! {
                biased;

                event = self.event_bus.1.recv() => {
                    match event.unwrap() {
                        RuntimeEvent::DriveMotion(motion_event) => {
                            let mut motion_device = self.motion_device.lock().await;
                            motion_device.actuate(motion_event).await;

                        }
                        RuntimeEvent::Shutdown => break,
                    }
                    // TODO: handle err.
                }

                _ = wait => self.idle_tlime().await,
            };
        }

        // Cancel all async tasks.
        for handle in &self.task_pool {
            handle.abort();
        }
    }
}

impl<A, K> Runtime<A, K>
where
    K: Operand + 'static,
{
    pub(super) fn spawn_input_device<C: InputDevice + 'static>(
        &mut self,
        input_device: DeviceDescriptor<C>,
    ) {
        let dispatcher = self.dispatch();
        let operand = self.operand.clone();

        self.spawn(async move {
            loop {
                if let Some(input) = input_device.lock().await.next().await {
                    if let Ok(motion) = operand.try_from_input_device(input) {
                        if let Err(_) = dispatcher.motion(motion).await {
                            warn!("Input event terminated without completion");
                            return;
                        }
                    }
                }
            }
        });
    }
}

impl<A, K> Runtime<A, K>
where
    A: MotionDevice,
    K: Operand + 'static,
{
    pub(super) fn spawn_program_queue(&mut self) {
        let dispatcher = self.dispatch();
        let operand = self.operand.clone();

        // TODO:
        let mut metric_devices = self.metric_devices.clone();

        // Move ownership of receiver to program queue thread.
        let mut receiver = self.program_queue.1.take().unwrap();

        self.spawn(async move {
            while let Some(id) = receiver.recv().await {
                let mut program = operand.fetch_program(id);

                info!("Start new program");

                let mut ctx = Context::default();
                program.boot(&mut ctx);

                // Loop until this program reaches its termination condition. If
                // the program does not terminate we'll run forever.
                while !program.can_terminate(&mut ctx) {
                    // FUTURE: The program progression is locked as long as we're waiting here.
                    for metric_device in metric_devices.iter_mut() {
                        match metric_device.lock().await.next().await {
                            Some((id, value)) => {
                                program.push(id as u32, value, &mut ctx);
                            }
                            None => {}
                        }
                    }

                    // Query the operand program for the next motion step.
                    if let Some(motion) = program.step(&mut ctx) {
                        if let Err(_) = dispatcher.motion(motion).await {
                            warn!("Program terminated without completion");
                            return;
                        }
                    }
                }

                // Execute an optional last action before program termination.
                if let Some(motion) = program.term_action(&mut ctx) {
                    if let Err(_) = dispatcher.motion(motion).await {
                        warn!("Program terminated without completion");
                        return;
                    }
                }

                info!("Program terminated");
            }
        });
    }
}
