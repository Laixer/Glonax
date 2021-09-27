use glonax_core::operand::Operand;

use crate::{
    device::{
        discover_instances, probe_claim_io_device, Gamepad, Inertial, IoDevice, IoDeviceProfile,
        MotionDevice,
    },
    runtime::{self, RuntimeSettings},
    workspace::Workspace,
    Config, Runtime,
};

/// Runtime builder.
///
/// The runtime builder is a convenient wrapper around the runtime core. It
/// creates and configures the core based on the global config and current
/// environment. It then presents the caller with a simple method to launch
/// the runtime loop.
///
/// The runtime builder *must* be used to construct a runtime.
pub struct Builder<'a, M, K> {
    /// Current application configuration.
    config: &'a Config,
    /// Current application workspace.
    #[allow(dead_code)]
    workspace: Workspace,
    /// Runtime core.
    runtime: Runtime<M, K>,
}

impl<'a, M: 'static + Send, K> Builder<'a, M, K>
where
    M: IoDevice + MotionDevice,
    M::DeviceProfile: IoDeviceProfile,
    K: Operand + 'static,
{
    /// Construct runtime service from configuration.
    ///
    /// Note that this method is certain to block.
    pub(crate) async fn from_config(config: &'a Config) -> super::Result<Builder<'a, M, K>> {
        Ok(Self {
            config,
            workspace: Workspace::new(&config.workspace)?,
            runtime: Self::bootstrap(config).await?,
        })
    }

    /// Construct the runtime core.
    ///
    /// The runtime core is created and initialized by the configuration.
    /// Any errors are fatal errors at this point.
    async fn bootstrap(config: &'a Config) -> super::Result<Runtime<M, K>> {
        use tokio::sync::mpsc;

        let mut device_manager = runtime::DeviceManager::new();

        let motion_device_unclaimed = discover_instances::<M>(&mut device_manager).await;

        let motion_device = match motion_device_unclaimed.into_iter().nth(0) {
            Some(motion_device) => motion_device,
            None => return Err(super::Error::MotionDeviceNotFound),
        };

        let program_queue = mpsc::channel(config.program_queue);

        let mut rt = Runtime {
            operand: K::default(),
            motion_device,
            metric_devices: vec![],
            event_bus: mpsc::channel(config.event_queue),
            program_queue: (program_queue.0, Some(program_queue.1)),
            settings: RuntimeSettings::from(config),
            task_pool: vec![],
            device_manager,
        };

        for metric_device in discover_instances::<Inertial>(&mut rt.device_manager).await {
            rt.metric_devices.push(metric_device);
        }

        Ok(rt)
    }

    async fn enable_term_shutdown(&self) {
        info!("Enable signals shutdown");

        let dispatcher = self.runtime.dispatch();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();

            info!("Termination requested");

            dispatcher.gracefull_shutdown().await.unwrap();
        });
    }

    async fn enable_autopilot(&mut self) {
        info!("Enable autopilot");

        self.runtime.spawn_program_queue();
    }

    async fn enable_input(&mut self) {
        info!("Enable input device(s)");

        let mut host_iface = crate::device::host::HostInterface::new();
        for mut device_claim in host_iface.elect::<<Gamepad as IoDevice>::DeviceProfile>() {
            trace!(
                "Elected claim: {}",
                device_claim.as_path().to_str().unwrap()
            );

            match probe_claim_io_device::<Gamepad>(&mut device_claim).await {
                Ok(input_device) => {
                    if device_claim.is_claimed() {
                        self.runtime
                            .device_manager
                            .register_device(input_device.clone());
                        self.runtime.spawn_input_device(input_device);
                        break;
                    }
                }
                Err(_) => {} // TODO: Only ignore NoSuchDevice.
            }
        }
    }

    /// Configure any optional runtime services.
    ///
    /// These runtime services depend on the application configuration.
    async fn config_services(&mut self) -> self::runtime::Result {
        // Enable shutdown service if configured.
        if self.config.enable_term_shutdown {
            self.enable_term_shutdown().await;
        }

        // Enable autopilot service if configured.
        if self.config.enable_autopilot {
            self.enable_autopilot().await;
        }

        // Enable input service if configured.
        if self.config.enable_input {
            self.enable_input().await;
        }

        Ok(())
    }

    /// Spawn the runtime service.
    ///
    /// This method consumes the runtime service.
    pub async fn spawn(mut self) -> self::runtime::Result {
        self.config_services().await?;

        // TODO: This is only for testing.
        self.runtime.program_queue.0.send(601).await.unwrap();

        self.runtime.run().await;

        Ok(())
    }
}
