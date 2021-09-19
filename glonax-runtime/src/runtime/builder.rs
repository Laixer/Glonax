use glonax_core::operand::Operand;

use crate::{
    device::{self, Device, Gamepad, Inertial, IoDevice, MotionDevice},
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

impl<'a, M: 'static, K> Builder<'a, M, K>
where
    M: IoDevice + MotionDevice,
    K: Operand + 'static,
{
    /// Construct runtime service from configuration.
    ///
    /// Note that this method is certain to block.
    pub fn from_config(config: &'a Config) -> super::Result<Self> {
        Ok(Self {
            config,
            workspace: Workspace::new(&config.workspace),
            runtime: Self::bootstrap(config)?,
        })
    }

    /// Create and probe the IO device.
    fn probe_io_device<D: IoDevice>(
        path: &String,
    ) -> device::Result<std::sync::Arc<std::sync::Mutex<D>>> {
        let mut io_device = D::from_path(path)?;

        debug!("Probe io device '{}' from path {}", io_device.name(), path);

        io_device.probe()?;

        info!("Device '{}' is online", io_device.name());

        Ok(std::sync::Arc::new(std::sync::Mutex::new(io_device)))
    }

    /// Construct the runtime core.
    ///
    /// The runtime core is created and initialized by the configuration.
    /// Any errors are fatal errors at this point.
    fn bootstrap(config: &'a Config) -> super::Result<Runtime<M, K>> {
        let motion_device = Self::probe_io_device::<M>(&config.motion_device)
            .map_err(|e| super::Error::Device(e))?;

        let program_queue = tokio::sync::mpsc::channel(config.program_queue);

        let mut rt = Runtime {
            operand: K::default(),
            motion_device: motion_device.clone(),
            metric_devices: vec![],
            event_bus: tokio::sync::mpsc::channel(64),
            program_queue: (program_queue.0, Some(program_queue.1)),
            settings: RuntimeSettings::from(config),
            task_pool: vec![],
            device_manager: runtime::DeviceManager::new(),
        };
        rt.device_manager.register_device(motion_device);

        for device in &config.metric_devices {
            match Self::probe_io_device::<Inertial>(device) {
                Ok(imu_device) => {
                    rt.metric_devices.push(imu_device.clone());
                    rt.device_manager.register_device(imu_device);
                }
                Err(e) => {
                    return Err(self::runtime::Error::Device(e));
                }
            }
        }

        Ok(rt)
    }

    async fn config_services(&mut self) -> self::runtime::Result {
        if self.config.enable_term_shutdown {
            info!("Enable signals shutdown");

            let dispatcher = self.runtime.dispatch();

            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();

                info!("Termination requested");

                dispatcher.gracefull_shutdown().await.unwrap();
            });
        }

        if self.config.enable_autopilot {
            info!("Enable autopilot");

            self.runtime.spawn_program_queue();
        }

        if self.config.enable_command {
            info!("Enable input device(s)");

            let mut gamepad = Gamepad::new();

            debug!("Probe '{}' device", gamepad.name());

            gamepad.probe().unwrap(); // TODO

            info!("Device '{}' is online", gamepad.name());

            self.runtime.spawn_command_device(gamepad);
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
