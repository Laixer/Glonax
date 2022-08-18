use std::time::Duration;

use crate::{
    config::Configurable,
    core::{Identity, Tracer},
    device::{Gateway, Hcu, Mecu, MotionDevice, Sink, Vecu},
    runtime, Runtime,
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
pub(crate) struct Builder<K> {
    /// Core device.
    core_device: Box<dyn crate::device::CoreDevice>,
    /// Runtime core.
    runtime: Runtime<K>,
}

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

        let mut gateway_device = Gateway::new(&config.global().interface);

        if tokio::time::timeout(Duration::from_secs(1), gateway_device.wait_online())
            .await
            .is_err()
        {
            return Err(super::Error::NetworkTimeout);
        }

        info!("Control network is online");

        let motion_device = if config.global().enable_motion {
            let motion_device = gateway_device.new_gateway_device::<Hcu>();

            Box::new(motion_device) as Box<dyn MotionDevice>
        } else {
            Box::new(Sink::new())
        };

        let signal_manager = crate::signal::SignalManager::new();

        // TODO: Should be optional, check arg.
        let signal_device = Mecu::new(signal_manager.pusher());
        gateway_device.subscribe(Box::new(signal_device));

        // TODO: Should be optional, check arg.
        gateway_device.new_gateway_device::<Vecu>();

        let runtime = Runtime {
            operand: K::from_config(config),
            motion_device,
            shutdown: broadcast::channel(1),
            signal_manager,
            tracer: runtime::CsvTracer::from_path(std::path::Path::new("/tmp/")),
        };

        Ok(Self {
            core_device: Box::new(gateway_device),
            runtime,
        })
    }

    pub(crate) fn enable_term_shutdown(self) -> Self {
        info!("Enable signals shutdown");

        let sender = self.runtime.shutdown.0.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();

            info!("Termination requested");

            sender.send(()).unwrap();
        });

        self
    }

    pub fn build_with_core_service(self) -> Runtime<K> {
        info!("Start core service");

        let mut core_device = self.core_device;

        tokio::task::spawn(async move { while core_device.next().await.is_ok() {} });

        self.runtime
    }

    #[inline]
    pub fn build(self) -> Runtime<K> {
        self.runtime
    }
}
