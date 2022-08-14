use std::time::Duration;

use crate::{
    core::{motion::Motion, Identity, TraceWriter, Tracer},
    device::{DeviceDescriptor, Gamepad, Gateway, Hcu, Mecu, MotionDevice, Sink, Vecu},
    runtime, Config, Runtime,
};

use super::{operand::ProgramFactory, Operand};

/// Runtime builder.
///
/// The runtime builder is a convenient wrapper around the runtime core. It
/// creates and configures the core based on the global config and current
/// environment. It then presents the caller with a simple method to launch
/// the runtime loop.
///
/// The runtime builder *must* be used to construct a runtime.
pub struct Builder<'a, K, R> {
    /// Current application configuration.
    config: &'a Config,
    /// Current application workspace.
    #[allow(dead_code)]
    lock: std::fs::File,
    /// Runtime core.
    runtime: Runtime<K, R>,
}

impl<'a, K, R> Builder<'a, K, R>
where
    K: Operand + Identity + ProgramFactory + 'static,
    R: Tracer + 'static,
    R::Instance: TraceWriter + Send + 'static,
{
    /// Construct runtime service from configuration.
    ///
    /// Note that this method is certain to block.
    pub(crate) async fn from_config(config: &'a Config) -> super::Result<Builder<'a, K, R>> {
        crate::workspace::setup_if_not_exists(&config.workspace);

        Ok(Self {
            config,
            lock: crate::workspace::lock(&config.workspace)?,
            runtime: Self::bootstrap(config).await?,
        })
    }

    /// Construct the runtime core.
    ///
    /// The runtime core is created and initialized by the configuration.
    /// Any errors are fatal errors at this point.
    async fn bootstrap(config: &'a Config) -> super::Result<Runtime<K, R>> {
        use tokio::sync::{broadcast, mpsc};

        info!("{}", K::intro());

        let session = runtime::RuntimeSession::new(&config.workspace);

        info!("Runtime session ID: {}", session.id);

        crate::workspace::store_value(&config.workspace, "session", session.id);

        let tracer = R::from_path(&session.path);

        let mut device_manager = crate::device::DeviceManager::new();

        let gateway: DeviceDescriptor<Gateway> = match device_manager
            .register_device_driver_first(|a, _| Gateway::new(a.as_str()))
            .await
        {
            Some(gateway) => gateway,
            None => return Err(super::Error::CoreDeviceNotFound),
        };

        if tokio::time::timeout(Duration::from_secs(1), gateway.lock().await.wait_online())
            .await
            .is_err()
        {
            return Err(super::Error::NetworkTimeout);
        }

        info!("Controller network is active");

        let motion_device = if config.enable_motion {
            gateway.lock().await.new_gateway_device::<Hcu>();

            let motion_device =
                device_manager.register_driver(gateway.lock().await.new_gateway_device::<Hcu>());
            gateway.lock().await.subscribe(motion_device.clone());
            motion_device as DeviceDescriptor<dyn MotionDevice>
        } else {
            device_manager.register_driver(Sink::new())
        };

        // let signal_tracer = tracer.instance("signal");

        let signal_manager = crate::signal::SignalManager::new();

        let signal_device = device_manager.register_driver(Mecu::new(signal_manager.pusher()));
        gateway.lock().await.subscribe(signal_device);

        let vehicle_device =
            device_manager.register_driver(gateway.lock().await.new_gateway_device::<Vecu>());

        gateway.lock().await.subscribe(vehicle_device);

        let program_queue = mpsc::channel(config.program_queue);

        let runtime = Runtime {
            operand: K::from_config(config),
            core_device: gateway,
            motion_device,
            motion: tokio::sync::mpsc::channel(32),
            shutdown: broadcast::channel(1),
            program_queue: (program_queue.0, Some(program_queue.1)),
            signal_manager,
            task_pool: vec![],
            device_manager,
            session,
            tracer,
        };

        Ok(runtime)
    }

    async fn enable_term_shutdown(&self) {
        info!("Enable signals shutdown");

        let sender = self.runtime.shutdown.0.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();

            info!("Termination requested");

            sender.send(()).unwrap();
        });
    }

    async fn spawn_motion_tracer(&self) {
        info!("Start motion tracer");

        self.runtime.spawn_motion_tracer();
    }

    async fn enable_autopilot(&mut self) {
        info!("Enable autopilot");

        self.runtime.spawn_program_queue();
    }

    /// Enable input devices to control the machine.
    async fn enable_input(&mut self) {
        info!("Enable input device(s)");

        match self
            .runtime
            .device_manager
            .try_register_device_driver_first::<Gamepad>(Duration::from_millis(100))
            .await
        {
            Some(input_device) => self.runtime.spawn_input_device(input_device),
            None => warn!("Input device not found"),
        };
    }

    /// Configure any optional runtime services.
    ///
    /// These runtime services depend on the application configuration.
    async fn config_services(&mut self) -> runtime::Result {
        // Enable terminal shutdown service.
        self.enable_term_shutdown().await;

        self.runtime.device_manager.startup();

        // Spawn the core device.
        self.runtime.spawn_core_device();

        self.spawn_motion_tracer().await;

        // Enable autopilot service if configured.
        if self.config.enable_autopilot {
            self.enable_autopilot().await;
        } else {
            info!("Autopilot not enabled");
        }

        // Enable input service if configured.
        if self.config.enable_input {
            self.enable_input().await;
        } else {
            info!("Input not enabled");
        }

        if !self.config.enable_motion {
            info!("Motion not enabled");
        }

        Ok(())
    }

    /// Instruct the runtime to execute startup events.
    ///
    /// Startup events will be queued and run on the runtime core before any
    /// other events are executed.
    async fn startup_events(&self) {
        info!("Running startup events");

        self.runtime
            .motion_dispatch()
            .send(Motion::StopAll)
            .await
            .ok(); // TODO: Handle result
    }

    /// Validate the runtime setup and exit.
    ///
    /// This method consumes the runtime service.
    pub async fn validate(mut self) -> runtime::Result {
        self.config_services().await?;

        Ok(())
    }

    /// Spawn the runtime service.
    ///
    /// This method consumes the runtime service.
    pub async fn spawn(mut self) -> runtime::Result {
        self.config_services().await?;

        self.startup_events().await;

        if let Some(id) = self.config.program_id {
            self.runtime
                .program_queue
                .0
                .send((id, vec![]))
                .await
                .unwrap();
        } else {
            // TODO: This is only for testing.
            // Queue the drive program
            self.runtime
                .program_queue
                .0
                .send((603, vec![-1.73, 1.01]))
                .await
                .unwrap();

            self.runtime
                .program_queue
                .0
                .send((603, vec![-1.31, 0.87]))
                .await
                .unwrap();

            self.runtime
                .program_queue
                .0
                .send((603, vec![-0.56, 0.74]))
                .await
                .unwrap();

            self.runtime
                .program_queue
                .0
                .send((603, vec![-0.19, 0.46]))
                .await
                .unwrap();

            self.runtime
                .program_queue
                .0
                .send((603, vec![-0.82, 0.40]))
                .await
                .unwrap();

            self.runtime
                .program_queue
                .0
                .send((603, vec![-1.77, 0.36]))
                .await
                .unwrap();

            self.runtime
                .program_queue
                .0
                .send((603, vec![-2.09, 0.63]))
                .await
                .unwrap();
        }

        self.runtime.run().await;

        Ok(())
    }
}
