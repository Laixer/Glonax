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
    core_device: Gateway,
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

        debug!("Bind to interface {}", config.global().interface);

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

        let runtime = Runtime {
            operand: K::from_config(config),
            motion_device,
            shutdown: broadcast::channel(1),
            signal_manager: crate::signal::SignalManager::new(),
            tracer: runtime::CsvTracer::from_path(std::path::Path::new("/tmp/")),
        };

        Ok(Self {
            core_device: gateway_device,
            runtime,
        })
    }

    pub(crate) fn subscribe_metric_unit(mut self) -> Self {
        debug!("Subscribe M-ECU to gateway");

        let signal_device = Mecu::new(self.runtime.signal_manager.pusher());
        self.core_device.subscribe(signal_device);

        self
    }

    pub(crate) fn subscribe_vehicle_unit(mut self) -> Self {
        debug!("Subscribe V-ECU to gateway");

        self.core_device.new_gateway_device::<Vecu>();

        self
    }

    pub(crate) fn enable_term_shutdown(self) -> Self {
        debug!("Enable signals shutdown");

        let sender = self.runtime.shutdown.0.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();

            info!("Termination requested");

            sender.send(()).unwrap();
        });

        self
    }

    pub fn build_with_core_service(mut self) -> Runtime<K> {
        use crate::device::CoreDevice;

        info!("Start core service");

        tokio::task::spawn(async move { while self.core_device.next().await.is_ok() {} });

        self.runtime
    }

    #[inline]
    pub fn build(self) -> Runtime<K> {
        self.runtime
    }
}
