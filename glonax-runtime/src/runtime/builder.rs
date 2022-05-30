use std::time::Duration;

use glonax_core::{motion::Motion, Identity, TraceWriter, Tracer};

use crate::{
    device::{Gamepad, Inertial, IoDeviceProfile, MotionDevice, UserDevice},
    runtime::{self, RuntimeSettings},
    Config, Runtime,
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
pub struct Builder<'a, M, K, R> {
    /// Current application configuration.
    config: &'a Config,
    /// Current application workspace.
    #[allow(dead_code)]
    lock: std::fs::File,
    /// Runtime core.
    runtime: Runtime<M, K, R>,
}

impl<'a, M: 'static + Send, K, R> Builder<'a, M, K, R>
where
    M: MotionDevice + UserDevice,
    M::DeviceRuleset: IoDeviceProfile,
    K: Operand + Identity + 'static,
    R: Tracer,
    R::Instance: TraceWriter + Send + 'static,
{
    /// Construct runtime service from configuration.
    ///
    /// Note that this method is certain to block.
    pub(crate) async fn from_config(config: &'a Config) -> super::Result<Builder<'a, M, K, R>> {
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
    async fn bootstrap(config: &'a Config) -> super::Result<Runtime<M, K, R>> {
        use tokio::sync::mpsc;

        info!("{}", K::intro());

        // TODO: Maybe ref the config in the dev mgr
        let mut device_manager = crate::device::DeviceManager::new();

        // Locate one and only one motion device.
        let motion_device = match device_manager
            .observer()
            .scan_first::<M>(std::time::Duration::from_millis(500))
            .await
        {
            Some(motion_device) => motion_device,
            None => return Err(super::Error::MotionDeviceNotFound),
        };

        let session = runtime::RuntimeSession::new(&config.workspace);

        info!("Runtime session ID: {}", session.id);

        crate::workspace::store_value(&config.workspace, "session", session.id);

        let tracer = R::from_path(&session.path);

        let program_queue = mpsc::channel(config.program_queue);

        let mut runtime = Runtime {
            operand: K::default(),
            motion_device,
            metric_devices: vec![],
            event_bus: mpsc::channel(config.event_queue),
            program_queue: (program_queue.0, Some(program_queue.1)),
            settings: RuntimeSettings::from(config),
            task_pool: vec![],
            device_manager,
            session,
            tracer,
        };

        for metric_device in runtime
            .device_manager
            .observer()
            .scan::<Inertial>(std::time::Duration::from_millis(500))
            .await
        {
            runtime.metric_devices.push(metric_device);
        }

        Ok(runtime)
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

    /// Enable input devices to control the machine.
    async fn enable_input(&mut self) {
        info!("Enable input device(s)");

        match self
            .runtime
            .device_manager
            .observer()
            .scan_first::<Gamepad>(Duration::from_millis(250))
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

    /// Instruct the runtime to execute startup events.
    ///
    /// Startup events will be queued and run on the runtime core before any
    /// other events are executed.
    async fn startup_events(&self) {
        let dispatcher = self.runtime.dispatch();

        info!("Running startup events");

        dispatcher.motion(Motion::StopAll).await.unwrap();
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

        // TODO: This is only for testing.
        // Queue the drive program
        self.runtime
            .program_queue
            .0
            .send((700, vec![10.0]))
            .await
            .unwrap();

        // Queue the noop. No operation will run forever.
        // self.runtime.program_queue.0.send(900).await.unwrap();

        self.runtime.run().await;

        Ok(())
    }
}
