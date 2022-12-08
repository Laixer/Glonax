use crate::{config::Configurable, core::Identity, runtime::EventHub, RuntimeContext};

use super::Operand;

/// Runtime builder.
///
/// The runtime builder is a convenient wrapper around the runtime core. It
/// creates and configures the core based on the global config and current
/// environment. It then presents the caller with a simple method to launch
/// the runtime loop.
///
/// The runtime builder *must* be used to construct a runtime.
pub(crate) struct Builder<K>(RuntimeContext<K>);

impl<K: Operand + Identity> Builder<K> {
    /// Construct runtime service from configuration.
    ///
    /// Note that this method is certain to block.
    pub(crate) fn from_config(config: &impl Configurable) -> super::Result<Builder<K>> {
        use tokio::sync::broadcast;

        info!("{}", K::intro());

        Ok(Self(RuntimeContext {
            operand: K::from_config(config),
            shutdown: broadcast::channel(1),
            eventhub: EventHub::new(config.global()),
        }))
    }

    pub(crate) fn enable_term_shutdown(self) -> Self {
        debug!("Enable signals shutdown");

        let sender = self.0.shutdown.0.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();

            info!("Termination requested");

            sender.send(()).unwrap();
        });

        self
    }

    #[inline]
    pub fn build(self) -> RuntimeContext<K> {
        self.0
    }
}
