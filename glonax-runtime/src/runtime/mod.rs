use std::time::{Duration, Instant};

use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    core::{self, motion::Motion, time, Trace, TraceWriter, Tracer},
    device::{CoreDevice, DeviceDescriptor, DeviceManager, InputDevice, MotionDevice},
    Config,
};

pub mod operand;
mod trace;
pub use trace::CsvTracer;
pub use trace::NullTracer;

mod error;
pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

mod builder;
pub use self::builder::Builder;
use self::operand::{Operand, Parameter, ProgramFactory};

struct MotionChain<R>
where
    R: Tracer,
    R::Instance: TraceWriter + Send + 'static,
{
    trace: R::Instance,
    motion_device: DeviceDescriptor<dyn MotionDevice>,
}

impl<R> MotionChain<R>
where
    R: Tracer,
    R::Instance: TraceWriter + Send + 'static,
{
    pub fn new(motion_device: DeviceDescriptor<dyn MotionDevice>, tracer: &R) -> Self {
        Self {
            motion_device,
            trace: tracer.instance("motion"),
        }
    }

    pub async fn request(&mut self, motion: Motion) {
        motion.record(&mut self.trace, time::now());

        self.motion_device.lock().await.actuate(motion).await;
    }
}

#[derive(serde::Serialize)]
struct ProgramTrace {
    /// Timestamp of the trace.
    timestamp: u128,
    /// Program identifier.
    id: i32,
}

#[derive(Clone)]
pub struct RuntimeSession {
    /// Session ID.
    pub id: uuid::Uuid,
    /// Session path on disk.
    pub path: std::path::PathBuf,
}

impl RuntimeSession {
    /// Construct new runtime session.
    ///
    /// The session identifier is unique and valid for the duration of the
    /// session.
    ///
    /// The runtime session will create a directory on disk in the name
    /// of the session.
    pub(super) fn new(path: &std::path::Path) -> Self {
        use std::io::Write;

        let id = uuid::Uuid::new_v4();
        let path = crate::workspace::create_directory(path, &id);

        debug!("Session directory: {}", &path.to_str().unwrap());

        let mut bootstrap = std::fs::File::create(path.join("bootstrap")).unwrap();
        writeln!(bootstrap, "BOOT = 1").unwrap();

        Self { id, path }
    }
}

impl std::fmt::Display for RuntimeSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

pub(super) struct RuntimeSettings {
    //
}

impl From<&Config> for RuntimeSettings {
    fn from(_config: &Config) -> Self {
        Self {}
    }
}

pub struct Runtime<K, R> {
    /// Runtime operand.
    pub(super) operand: K,
    /// Core device that runs machine logic.
    pub(super) core_device: DeviceDescriptor<dyn CoreDevice>,
    /// The standard motion device.
    pub(super) motion_device: DeviceDescriptor<dyn MotionDevice>,
    /// Runtime event bus.
    pub(super) motion: (
        tokio::sync::mpsc::Sender<Motion>,
        tokio::sync::mpsc::Receiver<Motion>,
    ),
    /// Runtime event bus.
    pub(super) shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
    /// Program queue.
    pub(super) program_queue: (Sender<(i32, Parameter)>, Option<Receiver<(i32, Parameter)>>),
    /// Signal manager.
    pub(super) signal_manager: crate::signal::SignalManager,
    /// Runtime settings.
    pub(super) settings: RuntimeSettings,
    /// Task pool.
    pub(super) task_pool: Vec<JoinHandle<()>>,
    /// Device manager.
    pub(super) device_manager: DeviceManager,
    /// Runtime session.
    pub(super) session: RuntimeSession,
    /// Tracer used to record telemetrics.
    pub(super) tracer: R,
}

impl<K, R> Runtime<K, R> {
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

    pub(super) fn spawn_core_device(&mut self) {
        let core_device = self.core_device.clone();

        self.spawn(async move { while core_device.lock().await.next().await.is_ok() {} });
    }

    pub(super) fn spawn_motion_tracer(&self) {
        //
    }
}

impl<K, R> Runtime<K, R>
where
    K: Operand + ProgramFactory + 'static,
    R: Tracer + 'static,
    R::Instance: TraceWriter + Send + 'static,
{
    #[inline]
    fn motion_dispatch(&self) -> Sender<Motion> {
        self.motion.0.clone()
    }

    /// Start the runtime.
    ///
    /// The runtime will process the events from the event bus. The runtime
    /// should only every break out of the loop if shutdown was requested.
    pub(super) async fn run(&mut self) {
        let mut motion_chain = MotionChain::new(self.motion_device.clone(), &self.tracer);

        loop {
            tokio::select! {
                motion = self.motion.1.recv() => {
                    if let Some(motion) = motion {
                        motion_chain.request(motion).await;
                    }
                }
                _ = self.shutdown.1.recv() => {
                    break;
                }
            }
        }

        // Stop all motion before exit.
        motion_chain.request(Motion::StopAll).await;

        debug!("Abort running tasks");

        // Cancel all async tasks.
        for handle in &self.task_pool {
            handle.abort();
        }
    }

    pub(super) fn spawn_input_device<C: InputDevice + 'static>(
        &mut self,
        input_device: DeviceDescriptor<C>,
    ) {
        use crate::core::motion::ToMotion;

        let mut operand = self.operand.clone();
        let motion_dispatch = self.motion_dispatch();

        std::thread::spawn(move || {
            while let Ok(input) = input_device.blocking_lock().next() {
                if let Ok(motion) = operand.try_from_input_device(input) {
                    motion_dispatch
                        .blocking_send(motion.to_motion())
                        .expect("channel gone");
                    // TODO: Replace expect
                }
            }
        });
    }

    pub(super) fn spawn_program_queue(&mut self) {
        use crate::core::motion::ToMotion;

        let operand = self.operand.clone();
        let motion_dispatch = self.motion_dispatch();

        let runtime_session = self.session.clone();
        let signal_reader = self.signal_manager.reader();

        let mut receiver = self.program_queue.1.take().unwrap();

        let mut program_tracer = self.tracer.instance("program");

        self.spawn(async move {
            while let Some((id, params)) = receiver.recv().await {
                let mut program = match operand.fetch_program(id, params) {
                    Ok(program) => program,
                    Err(_) => {
                        warn!("Program {} was not registered with the operand", id);
                        continue;
                    }
                };

                info!("Start program: {}", id);

                program_tracer.write_record(ProgramTrace {
                    timestamp: core::time::now().as_millis(),
                    id,
                });

                motion_dispatch.send(Motion::ResumeAll).await.ok(); // TOOD: Handle result

                let mut ctx = operand::Context::new(signal_reader.clone(), runtime_session.clone());
                program.boot(&mut ctx);

                // Loop until this program reaches its termination condition. If
                // the program does not terminate we'll run forever.
                while !program.can_terminate(&mut ctx) {
                    // Deliberately slow down the program loop to limit CPU cycles.
                    // If the delay is small then this won't effect the program
                    // procession.
                    tokio::time::sleep(Duration::from_millis(1)).await;

                    let start_step_execute = Instant::now();

                    // FUTURE: Ensure the step is called *at least* once ever 50ms.
                    // Query the operand program for the next motion step. The
                    // entire thread is dedicated to the program therefore steps
                    // can take as long as they require.
                    if let Some(motion) = program.step(&mut ctx) {
                        motion_dispatch.send(motion.to_motion()).await.ok(); // TOOD: Handle result
                    }

                    ctx.step_count += 1;
                    ctx.last_step = start_step_execute;
                }

                // Execute an optional last action before program termination.
                if let Some(motion) = program.term_action(&mut ctx) {
                    motion_dispatch.send(motion.to_motion()).await.ok(); // TOOD: Handle result
                }

                // Stop all motion for safety.
                motion_dispatch.send(Motion::StopAll).await.ok(); // TOOD: Handle result

                info!("Program terminated");
            }
        });
    }
}
