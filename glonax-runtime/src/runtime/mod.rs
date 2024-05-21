mod error;

use crate::world::World;

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type IPCSender = std::sync::mpsc::Sender<crate::core::ObjectMessage>;
pub type IPCReceiver = std::sync::mpsc::Receiver<crate::core::ObjectMessage>;
pub type CommandSender = tokio::sync::mpsc::Sender<crate::core::Object>;
pub type CommandReceiver = tokio::sync::mpsc::Receiver<crate::core::Object>;

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
    fn setup(&mut self) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Teardown the service.
    ///
    /// This method is called once on shutdown and should be used to cleanup the service.
    fn teardown(&mut self) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Wait for IO event.
    ///
    /// This method is always called on a separate thread and
    /// should be used to wait for IO events. The method is optional
    /// and does not need to be implemented.
    fn wait_io(
        &mut self,
        _ipc_tx: IPCSender,
        _command_tx: CommandSender,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Tick the component on interval.
    ///
    /// This method is called in conjunction with other services
    /// and should therefore be non-blocking. The method is optional
    /// and does not need to be implemented.
    fn tick(&mut self, _command_tx: CommandSender) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn tick2(&mut self, _ipc_rx: std::rc::Rc<IPCReceiver>, _command_tx: CommandSender) {}

    fn on_command(
        &mut self,
        _object: &crate::core::Object,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }
}

struct ServiceDescriptor<S, C = crate::runtime::NullConfig>
where
    S: Service<C> + Send + Sync + 'static,
    C: Clone + Send + 'static,
{
    service: S,
    ipc_tx: IPCSender,
    command_tx: CommandSender,
    shutdown: tokio::sync::broadcast::Receiver<()>,
    phantom: std::marker::PhantomData<C>,
}

impl<S> ServiceDescriptor<S, crate::runtime::NullConfig>
where
    S: Service<crate::runtime::NullConfig> + Send + Sync + 'static,
{
    fn new(
        service: S,
        ipc_tx: IPCSender,
        command_tx: CommandSender,
        shutdown: tokio::sync::broadcast::Receiver<()>,
    ) -> Self {
        Self {
            service,
            ipc_tx,
            command_tx,
            shutdown,
            phantom: std::marker::PhantomData,
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
        ipc_tx: IPCSender,
        command_tx: CommandSender,
        shutdown: tokio::sync::broadcast::Receiver<()>,
    ) -> Self {
        Self {
            service: S::new(config.clone()),
            ipc_tx,
            command_tx,
            shutdown,
            phantom: std::marker::PhantomData,
        }
    }

    async fn setup(&mut self) {
        log::debug!("Setup runtime service '{}'", self.service.ctx());

        self.service.setup().await;
    }

    async fn teardown(&mut self) {
        log::debug!("Teardown runtime service '{}'", self.service.ctx());

        self.service.teardown().await;
    }

    async fn wait_io(&mut self) {
        tokio::select! {
            _ = async {
                loop {
                    self.service.wait_io(self.ipc_tx.clone(), self.command_tx.clone()).await;
                }
            } => {}
            _ = self.shutdown.recv() => {}
        }
    }

    async fn tick(&mut self, duration: std::time::Duration) {
        let mut interval = tokio::time::interval(duration);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        while self.shutdown.is_empty() {
            interval.tick().await;

            let tick_start = std::time::Instant::now();

            self.service.tick(self.command_tx.clone()).await;

            let tick_duration = tick_start.elapsed();
            log::trace!("Tick loop duration: {:?}", tick_duration);

            if tick_duration > duration {
                log::warn!("Tick loop delta is too high: {:?}", tick_duration);
            }
        }
    }

    async fn tick2(&mut self, duration: std::time::Duration, ipc_rx: IPCReceiver) {
        let mut interval = tokio::time::interval(duration);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        // Wrap the IPC in an arc to allow for cloning.
        let ipc_rx = std::rc::Rc::new(ipc_rx);

        while self.shutdown.is_empty() {
            interval.tick().await;

            let tick_start = std::time::Instant::now();

            self.service.tick2(ipc_rx.clone(), self.command_tx.clone());

            let tick_duration = tick_start.elapsed();
            log::trace!("Tick loop duration: {:?}", tick_duration);

            if tick_duration > duration {
                log::warn!("Tick loop delta is too high: {:?}", tick_duration);
            }
        }
    }

    async fn on_command(&mut self, mut command_rx: CommandReceiver) {
        tokio::select! {
            _ = async {
                while let Some(command) = command_rx.recv().await {
                    self.service.on_command(&command).await;
                }
            } => {}
            _ = self.shutdown.recv() => {}
        }
    }
}

use crate::core;

/// Represents the state of a machine.
///
/// The project refers to the machine as the entire system including
/// hardware, software, sensors and actuators.
#[derive(Default)]
pub struct Machine {
    /// Vehicle management system data.
    pub vms_signal: core::Host,
    /// Vehicle management system update.
    pub vms_signal_instant: Option<std::time::Instant>,

    /// Global navigation satellite system data.
    pub gnss_signal: core::Gnss,
    /// GNSS signal update.
    pub gnss_signal_instant: Option<std::time::Instant>,

    /// Engine signal.
    pub engine_signal: core::Engine,
    /// Engine state actual instant.
    pub engine_signal_instant: Option<std::time::Instant>,
    /// Engine command.
    pub engine_command: Option<core::Engine>,
    /// Engine state request instant.
    pub engine_command_instant: Option<std::time::Instant>,

    /// Motion signal.
    pub motion_signal: core::Motion,
    /// Motion signal instant.
    pub motion_signal_instant: Option<std::time::Instant>,
    /// Motion command.
    pub motion_command: Option<core::Motion>,
    /// Motion command instant.
    pub motion_command_instant: Option<std::time::Instant>,

    /// Control command.
    pub control_command: Option<core::Control>,
    /// Control command instant.
    pub control_command_instant: Option<std::time::Instant>,

    /// Encoder data.
    pub encoders: std::collections::HashMap<u8, f32>, // TODO: HACK: Temporary
    /// Encoder instant.
    pub encoders_instant: Option<std::time::Instant>, // TODO: HACK: Temporary

    /// Current program queue.
    pub program_command: std::collections::VecDeque<core::Target>,
}

// TODO: Move
/// Component context.
///
/// The component context is provided to each component on each tick. The
/// component context is used to communicate within the component pipeline.
pub struct ComponentContext {
    /// Machine state.
    pub machine: Machine,
    /// World state.
    pub world: World,
    /// Actuator values.
    pub actuators: std::collections::HashMap<u16, f32>, // TODO: Find another way to pass actuator errors around. Maybe via objects.
    /// Last tick.
    last_tick: std::time::Instant,
    /// Iteration count.
    iteration: u64,
}

impl ComponentContext {
    /// Retrieve the tick delta.
    pub fn delta(&self) -> std::time::Duration {
        self.last_tick.elapsed()
    }

    /// Retrieve the iteration count.
    #[inline]
    pub fn iteration(&self) -> u64 {
        self.iteration
    }

    /// Called after all components are ticked.
    pub(crate) fn post_tick(&mut self) {
        self.actuators.clear();
        self.last_tick = std::time::Instant::now();
        self.iteration += 1;
    }
}

impl Default for ComponentContext {
    fn default() -> Self {
        Self {
            machine: Machine::default(),
            world: World::default(),
            actuators: std::collections::HashMap::new(),
            last_tick: std::time::Instant::now(),
            iteration: 0,
        }
    }
}

pub trait InitComponent<Cnf: Clone> {
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    fn init(&self, ctx: &mut ComponentContext, ipc_rx: std::rc::Rc<IPCReceiver>);
}

pub trait PostComponent<Cnf: Clone> {
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    fn finalize(&self, ctx: &mut ComponentContext, command_tx: CommandSender);
}

pub trait Component<Cnf: Clone> {
    /// Construct a new component.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    // TODO: Remove `command_tx`
    /// Tick the component.
    ///
    /// This method will be called on each tick of the runtime.
    /// How often the runtime ticks is determined by the runtime configuration.
    fn tick(&mut self, ctx: &mut ComponentContext, command_tx: CommandSender);
}

/// Construct runtime service from configuration.
///
/// Note that this method is certain to block.
#[inline]
pub fn builder() -> self::Result<builder::Builder> {
    builder::Builder::new()
}

pub struct Runtime {
    /// IPC sender.
    ipc_tx: IPCSender,
    /// IPC receiver.
    ipc_rx: Option<IPCReceiver>,
    /// Command sender.
    command_tx: CommandSender,
    /// Command receiver.
    command_rx: Option<CommandReceiver>,
    /// Runtime tasks.
    tasks: Vec<tokio::task::JoinHandle<()>>, // TODO: Rename to task pool.
    /// Runtime event bus.
    shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
}

impl Runtime {
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
            self.ipc_tx.clone(),
            self.command_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        if self.shutdown.1.is_empty() {
            self.spawn(async move {
                service_descriptor.setup().await;
                service_descriptor.wait_io().await;
                service_descriptor.teardown().await;
            });
        }
    }

    /// Listen for command event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_command_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let command_rx = self.command_rx.take().unwrap();

        let mut service_descriptor = ServiceDescriptor::<S, _>::with_config(
            config,
            self.ipc_tx.clone(),
            self.command_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        if self.shutdown.1.is_empty() {
            self.spawn(async move {
                service_descriptor.setup().await;
                service_descriptor.on_command(command_rx).await;
                service_descriptor.teardown().await;
            });
        }
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
            self.ipc_tx.clone(),
            self.command_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        if self.shutdown.1.is_empty() {
            self.spawn(async move {
                service_descriptor.setup().await;
                service_descriptor.tick(duration).await;
                service_descriptor.teardown().await;
            });
        }
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
            self.ipc_tx.clone(),
            self.command_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        if self.shutdown.1.is_empty() {
            self.spawn(async move {
                service_descriptor.setup().await;
                service_descriptor.tick(duration).await;
                service_descriptor.teardown().await;
            });
        }
    }

    /// Run a service in the background.
    ///
    /// This method will run a service in the background. The service will be provided with a copy of
    /// the runtime configuration and a reference to the runtime.
    pub async fn run_interval<S>(&mut self, service: S, duration: std::time::Duration)
    where
        S: Service<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        let ipc_rx = self.ipc_rx.take().unwrap();

        let mut service_descriptor = ServiceDescriptor::new(
            service,
            self.ipc_tx.clone(),
            self.command_tx.clone(),
            self.shutdown.0.subscribe(),
        );

        if self.shutdown.1.is_empty() {
            service_descriptor.setup().await;
            service_descriptor.tick2(duration, ipc_rx).await;
            service_descriptor.teardown().await;
        }
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
