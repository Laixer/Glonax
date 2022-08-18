use crate::{
    core::{motion::ToMotion, time, Trace, TraceWriter, Tracer},
    device::MotionDevice,
};

pub mod operand;
mod trace;
pub use trace::CsvTracer;
pub use trace::NullTracer;

mod error;
pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

mod builder;
pub(crate) use self::builder::Builder;
use self::operand::Operand;

mod program;
pub use program::RuntimeProgram;

mod input;
pub use input::RuntimeInput;

// TODO: Move into builder.
struct MotionChain<'a, R>
where
    R: Tracer,
    R::Instance: TraceWriter + Send + 'static,
{
    trace: R::Instance,
    motion_device: &'a mut Box<dyn MotionDevice>,
}

impl<'a, R> MotionChain<'a, R>
where
    R: Tracer,
    R::Instance: TraceWriter + Send + 'static,
{
    pub fn new(motion_device: &'a mut Box<dyn MotionDevice>, tracer: &R) -> Self {
        Self {
            motion_device,
            trace: tracer.instance("motion"),
        }
    }

    pub async fn request<T: ToMotion>(&mut self, motion: T) {
        let mo = motion.to_motion();
        mo.record(&mut self.trace, time::now());

        self.motion_device.actuate(mo).await;
    }
}

// TODO: Rename to RuntimeContext
pub struct Runtime<K> {
    /// Runtime operand.
    pub(super) operand: K,
    /// The standard motion device.
    pub(super) motion_device: Box<dyn MotionDevice>,
    /// Runtime event bus.
    pub(super) shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
    /// Signal manager.
    pub(super) signal_manager: crate::signal::SignalManager,
    /// Tracer used to record telemetrics.
    pub(super) tracer: CsvTracer,
}
