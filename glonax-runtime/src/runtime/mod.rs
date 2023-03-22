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
