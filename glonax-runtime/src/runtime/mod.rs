mod error;

use crate::{core::Target, world::World, MachineState};

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

// TODO: Rename to CommandSender
pub type CommandSender = tokio::sync::mpsc::Sender<crate::core::Object>;
pub type CommandReceiver = tokio::sync::mpsc::Receiver<crate::core::Object>;
pub type SharedOperandState = std::sync::Arc<tokio::sync::RwLock<crate::Operand>>;

pub mod builder;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NullConfig;

pub struct ServiceContext {
    /// Service name.
    name: String,
    /// Service address.
    address: Option<String>,
}

impl ServiceContext {
    /// Construct a new service context.
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            address: None,
        }
    }

    /// Construct a new service context with address.
    pub fn with_address(name: impl ToString, address: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            address: Some(address.to_string()),
        }
    }
}

impl std::fmt::Display for ServiceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(address) = &self.address {
            write!(f, "{} on {}", self.name, address)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

pub trait Service<Cnf> {
    // TODO: Add instance to new
    /// Construct a new component.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    /// Get the service context.
    fn ctx(&self) -> ServiceContext {
        ServiceContext {
            name: std::any::type_name::<Self>().to_string(),
            address: None,
        }
    }

    /// Setup the service.
    ///
    /// This method is called once on startup and should be used to initialize the service.
    fn setup(
        &mut self,
        _runtime_state: SharedOperandState,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Teardown the service.
    ///
    /// This method is called once on shutdown and should be used to cleanup the service.
    fn teardown(
        &mut self,
        _runtime_state: SharedOperandState,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Wait for IO event.
    ///
    /// This method is always called on a separate thread and
    /// should be used to wait for IO events. The method is optional
    /// and does not need to be implemented.
    fn wait_io(
        &mut self,
        _runtime_state: SharedOperandState,
        _command_tx: CommandSender,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Tick the component on interval.
    ///
    /// This method is called in conjunction with other services
    /// and should therefore be non-blocking. The method is optional
    /// and does not need to be implemented.
    fn tick(
        &mut self,
        _runtime_state: SharedOperandState,
        _command_tx: CommandSender,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn on_command(
        &mut self,
        _runtime_state: SharedOperandState,
        _object: &crate::core::Object,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }
}

pub trait Component<Cnf: Clone> {
    // TODO: Add instance to new
    /// Construct a new component.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    /// Tick the component.
    ///
    /// This method will be called on each tick of the runtime.
    /// How often the runtime ticks is determined by the runtime configuration.
    fn tick(
        &mut self,
        ctx: &mut ComponentContext,
        state: &mut MachineState,
        command_tx: CommandSender,
    );
}

struct ServiceDescriptor<S, C = crate::runtime::NullConfig>
where
    S: Service<C> + Send + Sync + 'static,
    C: Clone + Send + 'static,
{
    service: S,
    _config: C,
    operand: std::sync::Arc<tokio::sync::RwLock<crate::Operand>>,
    command_tx: CommandSender,
    shutdown: tokio::sync::broadcast::Receiver<()>,
}

impl<S> ServiceDescriptor<S, crate::runtime::NullConfig>
where
    S: Service<crate::runtime::NullConfig> + Send + Sync + 'static,
{
    fn new(
        service: S,
        operand: std::sync::Arc<tokio::sync::RwLock<crate::Operand>>,
        command_tx: CommandSender,
        shutdown: tokio::sync::broadcast::Receiver<()>,
    ) -> Self {
        Self {
            service,
            _config: crate::runtime::NullConfig,
            operand,
            command_tx,
            shutdown,
        }
    }
}

impl<S, C> ServiceDescriptor<S, C>
where
    S: Service<C> + Send + Sync + 'static,
    C: Clone + Send + 'static,
{
    fn with_config(
        config: C,
        operand: std::sync::Arc<tokio::sync::RwLock<crate::Operand>>,
        command_tx: CommandSender,
        shutdown: tokio::sync::broadcast::Receiver<()>,
    ) -> Self {
        Self {
            service: S::new(config.clone()),
            _config: config,
            operand,
            command_tx,
            shutdown,
        }
    }

    async fn setup(&mut self) {
        log::debug!("Setup runtime service '{}'", self.service.ctx());

        self.service.setup(self.operand.clone()).await;
    }

    async fn teardown(&mut self) {
        log::debug!("Teardown runtime service '{}'", self.service.ctx());

        self.service.teardown(self.operand.clone()).await;
    }

    async fn wait_io(&mut self) {
        log::debug!("Wait on IO runtime service '{}'", self.service.ctx());

        tokio::select! {
            _ = async {
                loop {
                    self.service.wait_io(self.operand.clone(), self.command_tx.clone()).await;
                }
            } => {}
            _ = self.shutdown.recv() => {}
        }
    }

    async fn tick(&mut self, duration: std::time::Duration) {
        log::debug!("Tick runtime service '{}'", self.service.ctx());

        while self.shutdown.is_empty() {
            tokio::time::sleep(duration).await;
            self.service
                .tick(self.operand.clone(), self.command_tx.clone())
                .await;
        }
    }

    async fn on_command(&mut self, mut command_rx: CommandReceiver) {
        log::debug!("Wait on command runtime service '{}'", self.service.ctx());

        tokio::select! {
            _ = async {
                while let Some(signal) = command_rx.recv().await {
                    self.service.on_command(self.operand.clone(), &signal).await;
                }
            } => {}
            _ = self.shutdown.recv() => {}
        }
    }
}

// TODO: Move
/// Component context.
///
/// The component context is provided to each component on each tick. The
/// component context is used to communicate within the component pipeline.
pub struct ComponentContext {
    /// World.
    pub world: World,
    /// Current target.
    pub target: Option<Target>,
    /// Actuator values.
    pub actuators: std::collections::HashMap<u16, f32>, // TODO: Find another way to pass actuator errors around.
    /// Last tick.
    last_tick: std::time::Instant,
}

impl ComponentContext {
    /// Retrieve the tick delta.
    pub fn delta(&self) -> std::time::Duration {
        self.last_tick.elapsed()
    }

    /// Called after all components are ticked.
    pub(crate) fn post_tick(&mut self) {
        self.actuators.clear();
        self.last_tick = std::time::Instant::now();
    }
}

impl Default for ComponentContext {
    fn default() -> Self {
        Self {
            world: World::default(),
            target: None,
            actuators: std::collections::HashMap::new(),
            last_tick: std::time::Instant::now(),
        }
    }
}

/// Construct runtime service from configuration and instance.
///
/// Note that this method is certain to block.
#[inline]
pub fn builder<Cnf: Clone>(
    config: &Cnf,
    instance: crate::core::Instance,
) -> self::Result<builder::Builder<Cnf>> {
    builder::Builder::new(config, instance)
}

pub struct Runtime<Conf> {
    /// Runtime configuration.
    #[allow(dead_code)]
    config: Conf, // TODO: Remove config.
    /// Instance.
    #[allow(dead_code)]
    instance: crate::core::Instance, // TODO: Remove instance.
    /// Glonax operand.
    operand: SharedOperandState, // TODO: Generic, TODO: Remove instance from operand.
    /// Motion command sender.
    motion_tx: CommandSender,
    /// Motion command receiver.
    motion_rx: Option<CommandReceiver>,
    /// Runtime tasks.
    tasks: Vec<tokio::task::JoinHandle<()>>, // TODO: Rename to task pool.
    /// Runtime event bus.
    shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
}

impl<Cnf: Clone + Send + 'static> Runtime<Cnf> {
    /// Spawns a future onto the runtime's executor.
    ///
    /// This method spawns a future onto the runtime's executor, allowing it to run in the background.
    /// The future must implement the `Future` trait with an output type of `()`, and it must also be `Send` and `'static`.
    fn spawn<F: std::future::Future<Output = ()> + Send + 'static>(&mut self, f: F) {
        self.tasks.push(tokio::spawn(f));
    }

    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let mut service_descriptor = ServiceDescriptor::<S, _>::with_config(
            config,
            self.operand.clone(),
            self.motion_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        self.spawn(async move {
            service_descriptor.setup().await;
            service_descriptor.wait_io().await;
            service_descriptor.teardown().await;
        });
    }

    /// Listen for signal event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_signal_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let command_rx = self.motion_rx.take().unwrap();

        let mut service_descriptor = ServiceDescriptor::<S, _>::with_config(
            config,
            self.operand.clone(),
            self.motion_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        self.spawn(async move {
            service_descriptor.setup().await;
            service_descriptor.on_command(command_rx).await;
            service_descriptor.teardown().await;
        });
    }

    /// Schedule a component to run in the background.
    ///
    /// This method will schedule a component to run in the background. On each tick, the component
    /// will be provided with a component context and a mutable reference to the runtime state.
    pub fn schedule_service<S, C>(&mut self, config: C, duration: std::time::Duration)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let mut service_descriptor = ServiceDescriptor::<S, _>::with_config(
            config,
            self.operand.clone(),
            self.motion_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        self.spawn(async move {
            service_descriptor.setup().await;
            service_descriptor.tick(duration).await;
            service_descriptor.teardown().await;
        });
    }

    /// Schedule a component to run in the background with default configuration.
    ///
    /// This method will schedule a component to run in the background. On each tick, the component
    /// will be provided with a component context and a mutable reference to the runtime state.
    pub fn schedule_service_default<S>(&mut self, duration: std::time::Duration)
    where
        S: Service<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        let mut service_descriptor = ServiceDescriptor::<S, _>::new(
            S::new(crate::runtime::NullConfig),
            self.operand.clone(),
            self.motion_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        self.spawn(async move {
            service_descriptor.setup().await;
            service_descriptor.tick(duration).await;
            service_descriptor.teardown().await;
        });
    }

    /// Run a service in the background.
    ///
    /// This method will run a service in the background. The service will be provided with a copy of
    /// the runtime configuration and a reference to the runtime.
    pub async fn run_interval<S>(&mut self, service: S, duration: std::time::Duration)
    where
        S: Service<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        let mut service_descriptor = ServiceDescriptor::new(
            service,
            self.operand.clone(),
            self.motion_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        service_descriptor.setup().await;
        service_descriptor.tick(duration).await;
        service_descriptor.teardown().await;
    }

    /// Wait for the runtime to shutdown.
    ///
    /// This method will block until the runtime is shutdown.
    pub async fn wait_for_shutdown(&mut self) {
        self.shutdown.1.recv().await.ok();
    }

    /// Wait for all tasks to complete.
    ///
    /// This method will block until all tasks are completed.    
    pub async fn wait_for_tasks(&mut self) {
        for task in self.tasks.drain(..) {
            task.await.ok();
        }
    }
}
