mod input;
mod motion;
mod operand;

pub use input::Scancode;
pub use motion::{Motion, NormalControl};
pub use operand::{Context, Operand, Program};

use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    device::{CommandDevice, MotionDevice},
    Config,
};

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
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RuntimeEvent>> {
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
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RuntimeEvent>> {
        self.0.send(RuntimeEvent::Shutdown).await
    }
}

pub struct RuntimeSettings {
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
        Self { timer_interval: 5 }
    }
}

/// Device manager.
///
/// The device manager keeps track of registered devices. Methods on the devices
/// are available on the device manager. On ever device method call we'll select
/// a new device from the manager. This allows the caller to automatically cycle
/// through all devices when the same method is called repeatedly.
///
/// By default devices selection is based on a simple round robin distribution.
pub(super) struct DeviceManager {
    device_list: Vec<std::sync::Arc<std::sync::Mutex<dyn crate::device::Device>>>,
    index: usize,
}

impl DeviceManager {
    /// Construct new device manager.
    pub(super) fn new() -> Self {
        Self {
            device_list: Vec::new(),
            index: 0,
        }
    }

    /// Register a device with the device manager.
    #[inline]
    pub(super) fn register_device(
        &mut self,
        device: std::sync::Arc<std::sync::Mutex<dyn crate::device::Device>>,
    ) {
        self.device_list.push(device)
    }

    /// Select the next device from the device list.
    ///
    /// Re-entering this method is likely to yield a different result.
    fn next(&mut self) -> &std::sync::Arc<std::sync::Mutex<dyn crate::device::Device>> {
        self.index += 1;
        self.device_list
            .get(self.index % self.device_list.len())
            .unwrap()
    }

    /// Call `idle_time` method on the next device.
    fn idle_time(&mut self) {
        if let Ok(mut device) = self.next().lock() {
            device.idle_time();
        }
    }
}

pub struct Runtime<A, K> {
    /// Runtime operand.
    pub(super) operand: K,
    /// Motion device.
    pub(super) motion_device: std::sync::Arc<std::sync::Mutex<A>>,
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
    /// The dispatcher is the input channel to the
    /// runtime event queue.
    #[inline]
    pub fn dispatch(&self) -> Dispatch {
        Dispatch(self.event_bus.0.clone())
    }
}

impl<A: MotionDevice, K> Runtime<A, K> {
    /// Start the runtime.
    ///
    /// The runtime will process the events from the
    /// event bus. The runtime should only every break
    /// out of the loop if shutdown was requested.
    pub async fn run(&mut self) {
        use tokio::time::{sleep, Duration};

        loop {
            let wait = sleep(Duration::from_secs(self.settings.timer_interval));

            tokio::select! {
                biased;

                event = self.event_bus.1.recv() => {
                    match event.unwrap() {
                        RuntimeEvent::DriveMotion(motion_event) => {
                            if let Ok(mut motion_device) = self.motion_device.lock() {
                                motion_device.actuate(motion_event)
                            }
                        }
                        RuntimeEvent::Shutdown => break,
                    }
                    // TODO: handle err.
                }

                _ = wait => self.device_manager.idle_time(),
            };
        }

        // TODO: Cancel all async tasks.

        for handle in &self.task_pool {
            handle.abort()
        }
    }
}

impl<A, K> Runtime<A, K>
where
    K: Operand + 'static,
{
    pub fn spawn_command_device<C: CommandDevice + Send + 'static>(
        &mut self,
        mut command_device: C,
    ) -> &mut Self {
        let dispatcher = self.dispatch();
        let operand = self.operand.clone();

        let task_handle = tokio::task::spawn(async move {
            loop {
                // FUTURE: We should be awaiting this.
                match command_device.next() {
                    Some(input) => {
                        if let Ok(motion) = operand.try_from_input_device(input) {
                            if let Err(_) = dispatcher.motion(motion).await {
                                warn!("Command event terminated without completion");
                                return;
                            }
                        }
                    }
                    None => tokio::time::sleep(tokio::time::Duration::from_millis(5)).await,
                }
            }
        });

        self.task_pool.push(task_handle);
        self
    }
}

impl<A: MotionDevice, K: Operand + 'static> Runtime<A, K> {
    pub fn spawn_program_queue<D>(
        &mut self,
        mut metric_devices: crate::device::Composer<Box<D>>,
    ) -> &mut Self
    where
        D: crate::device::MetricDevice + Send + Sync + 'static + ?Sized,
    {
        let dispatcher = self.dispatch();
        let operand = self.operand.clone();

        // Move ownership of receiver to program queue thread.
        let mut receiver = self.program_queue.1.take().unwrap();

        let task_handle = tokio::task::spawn(async move {
            while let Some(id) = receiver.recv().await {
                let mut program = operand.fetch_program(id);

                info!("Start new program");

                let mut ctx = Context::new();
                program.boot(&mut ctx);

                // Loop until this program reaches its termination condition. If
                // the program does not terminate we'll run forever.
                while !program.can_terminate(&mut ctx) {
                    for (idx, device) in &mut metric_devices.iter_mut() {
                        match device.next() {
                            Some(value) => {
                                program.push(idx.clone(), value, &mut ctx);
                            }
                            None => {}
                        }
                    }

                    if let Some(motion) = program.step(&mut ctx) {
                        if let Err(_) = dispatcher.motion(motion).await {
                            warn!("Program terminated without completion");
                            return;
                        }
                    }
                }

                if let Some(motion) = program.term_action(&mut ctx) {
                    if let Err(_) = dispatcher.motion(motion).await {
                        warn!("Program terminated without completion");
                        return;
                    }
                }

                info!("Program terminated");
            }
        });

        self.task_pool.push(task_handle);
        self
    }
}
