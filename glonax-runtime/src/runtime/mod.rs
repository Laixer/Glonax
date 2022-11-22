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

pub mod ecu;
pub mod exec;
pub mod input;

pub(super) struct MotionChain<'a, R, M>
where
    R: Tracer,
    R::Instance: TraceWriter + Send + 'static,
    M: MotionDevice,
{
    /// Motion trace instance.
    trace: R::Instance,
    /// Motion device.
    motion_device: &'a mut M,
    /// Whether or not to enable the motion device.
    motion_enabled: bool,
}

impl<'a, R, M> MotionChain<'a, R, M>
where
    R: Tracer,
    R::Instance: TraceWriter + Send + 'static,
    M: MotionDevice,
{
    pub fn new(motion_device: &'a mut M, tracer: &R) -> Self {
        Self {
            motion_device,
            trace: tracer.instance("motion"),
            motion_enabled: true,
        }
    }

    pub fn enable(mut self, is_enabled: bool) -> Self {
        self.motion_enabled = is_enabled;
        self
    }

    pub async fn request<T: ToMotion>(&mut self, motion: T) {
        let motion = motion.to_motion();
        motion.record(&mut self.trace, time::now());

        if self.motion_enabled {
            self.motion_device.actuate(motion).await;
        } else {
            debug!("Motion device is disabled: no motion commands will be issued");
        }
    }
}

pub struct RuntimeContext<K> {
    /// Runtime operand.
    pub(super) operand: K,
    /// Core device.
    pub(super) core_device: crate::device::Gateway,
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
