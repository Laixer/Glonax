use crate::{config::Configurable, RuntimeContext};

/// Runtime builder.
///
/// The runtime builder is a convenient wrapper around the runtime core. It
/// creates and configures the core based on the global config and current
/// environment. It then presents the caller with a simple method to launch
/// the runtime loop.
///
/// The runtime builder *must* be used to construct a runtime.
pub struct Builder(RuntimeContext);

impl Builder {
    /// Construct runtime service from configuration.
    ///
    /// Note that this method is certain to block.
    pub fn from_config(_config: &impl Configurable) -> super::Result<Self> {
        use tokio::sync::broadcast;

        Ok(Self(RuntimeContext {
            // operand: K::from_config(config),
            shutdown: broadcast::channel(1),
        }))
    }

    /// Listen for termination signal.
    ///
    /// This method will spawn a task that will listen for a termination signal.
    /// When the signal is received, the runtime will be shutdown.
    pub fn with_shutdown(self) -> Self {
        debug!("Enable shutdown signal");

        let sender = self.0.shutdown.0.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();

            info!("Termination requested");

            sender.send(()).unwrap();
        });

        self
    }

    /// Build the runtime.
    #[inline]
    pub fn build(self) -> RuntimeContext {
        self.0
    }
}
