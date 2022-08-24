use std::time::Duration;

use crate::{
    config::Configurable,
    core::{Identity, Tracer},
    device::Gateway,
    runtime, RuntimeContext,
};

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

impl<K> Builder<K>
where
    K: Operand + Identity,
{
    /// Construct runtime service from configuration.
    ///
    /// Note that this method is certain to block.
    pub(crate) async fn from_config(config: &impl Configurable) -> super::Result<Builder<K>> {
        use tokio::sync::broadcast;

        info!("{}", K::intro());

        debug!("Bind to interface {}", config.global().interface);

        let gateway_device = Gateway::new(&config.global().interface);

        if tokio::time::timeout(Duration::from_secs(1), gateway_device.wait_online())
            .await
            .is_err()
        {
            return Err(super::Error::NetworkTimeout);
        }

        info!("Control network is online");

        Ok(Self(RuntimeContext {
            operand: K::from_config(config),
            core_device: gateway_device,
            shutdown: broadcast::channel(1),
            signal_manager: crate::signal::SignalManager::new(),
            tracer: runtime::CsvTracer::from_path(std::path::Path::new("/tmp/")),
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
