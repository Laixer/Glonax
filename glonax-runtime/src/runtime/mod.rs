pub(super) mod operand;

mod error;

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub mod builder;

pub struct RuntimeContext {
    /// Runtime event bus.
    pub shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
}

impl RuntimeContext {
    /// Listen for shutdown signal.
    pub fn shutdown_signal(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown.0.subscribe()
    }
}
