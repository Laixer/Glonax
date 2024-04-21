mod error;

use crate::{
    core::{Instance, Target},
    world::World,
    MachineState,
};

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type MotionSender = tokio::sync::mpsc::Sender<crate::core::Motion>;
pub type MotionReceiver = tokio::sync::mpsc::Receiver<crate::core::Motion>;
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
    pub fn new(name: impl ToString, address: Option<impl ToString>) -> Self {
        Self {
            name: name.to_string(),
            address: address.map(|a| a.to_string()),
        }
    }
}

// TODO: Change to ServiceContext
pub struct ServiceErrorBuilder {
    name: String,
    address: String,
}

impl ServiceErrorBuilder {
    pub fn new(name: impl ToString, address: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            address: address.to_string(),
        }
    }

    pub fn io_error(&self, io_error: std::io::Error) -> ServiceError {
        ServiceError {
            name: self.name.clone(),
            address: self.address.clone(),
            io_error,
        }
    }
}

pub struct ServiceError {
    name: String,
    address: String,
    io_error: std::io::Error,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Service {} error: {}", self.name, self.io_error)
    }
}

impl std::fmt::Debug for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Service {} error: {:?}", self.name, self.io_error)
    }
}

impl std::error::Error for ServiceError {}

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
    fn setup(
        &mut self,
        _runtime_state: SharedOperandState,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Teardown the service.
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
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    // TODO: Replace the motion receiver with a generic event receiver
    fn on_event(
        &mut self,
        _runtime_state: SharedOperandState,
        _motion: &crate::core::Motion,
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

    /// Run the component once.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn once(&mut self, _ctx: &mut ComponentContext, _state: &mut MachineState) {}

    /// Tick the component.
    ///
    /// This method will be called on each tick of the runtime.
    /// How often the runtime ticks is determined by the runtime configuration.
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState);
}

/// Component context.
///
/// The component context is provided to each component on each tick. The
/// component context is used to communicate within the component pipeline.
pub struct ComponentContext {
    /// Motion command sender.
    motion_tx: tokio::sync::mpsc::Sender<crate::core::Motion>,
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
    /// Construct a new component context.
    pub fn new(motion_tx: tokio::sync::mpsc::Sender<crate::core::Motion>) -> Self {
        Self {
            motion_tx,
            world: World::default(),
            target: None,
            actuators: std::collections::HashMap::new(),
            last_tick: std::time::Instant::now(),
        }
    }

    /// Commit a motion command.
    pub fn commit(&mut self, motion: crate::core::Motion) {
        if let Err(e) = self.motion_tx.try_send(motion) {
            log::error!("Failed to send motion command: {}", e);
        }
    }

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
    config: Conf,
    /// Instance.
    instance: crate::core::Instance,
    /// Glonax operand.
    operand: SharedOperandState, // TODO: Generic, TODO: Remove instance from operand.
    /// Motion command sender.
    motion_tx: MotionSender,
    /// Motion command receiver.
    motion_rx: Option<MotionReceiver>,
    /// Runtime tasks.
    tasks: Vec<tokio::task::JoinHandle<()>>,
    /// Runtime event bus.
    shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
}

impl<Cnf: Clone + Send + 'static> Runtime<Cnf> {
    /// Listen for shutdown signal.
    #[inline]
    pub fn shutdown_signal(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown.0.subscribe()
    }

    // TODO: This is temporary
    pub fn motion_sender(&self) -> MotionSender {
        self.motion_tx.clone()
    }

    #[deprecated]
    pub fn schedule_motion_sink<Fut>(
        &mut self,
        service: impl FnOnce(Cnf, Instance, SharedOperandState, MotionReceiver) -> Fut + Send + 'static,
    ) where
        Fut: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
    {
        let config = self.config.clone();
        let instance = self.instance.clone();
        let operand = self.operand.clone();
        let motion_rx = self.motion_rx.take().unwrap();

        tokio::spawn(async move {
            if let Err(e) = service(config, instance, operand, motion_rx).await {
                log::error!("Failed to start motion service: {}", e);
            }
        });
    }

    //
    // Services
    //

    #[deprecated]
    pub fn schedule_io_func<Fut>(
        &self,
        service: impl FnOnce(Cnf, Instance, SharedOperandState, MotionSender) -> Fut + Send + 'static,
    ) where
        Fut: std::future::Future<Output = std::result::Result<(), ServiceError>> + Send + 'static,
    {
        let config = self.config.clone();
        let instance = self.instance.clone();
        let operand = self.operand.clone();
        let motion_tx = self.motion_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = service(config, instance, operand, motion_tx).await {
                log::error!(
                    "Failed to schedule '{}' at {}: {}",
                    e.name,
                    e.address,
                    e.io_error
                );
            }
        });
    }

    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone,
    {
        let mut service = S::new(config.clone());
        let ctx = service.ctx();

        let operand = self.operand.clone();
        let shutdown = self.shutdown.0.subscribe();

        if let Some(address) = ctx.address.clone() {
            log::debug!("Starting '{}' service on {}", ctx.name, address);
        } else {
            log::debug!("Starting '{}' service", ctx.name);
        }

        // TODO: Replace loop with tokio::select
        self.tasks.push(tokio::spawn(async move {
            service.setup(operand.clone()).await;
            while shutdown.is_empty() {
                service.wait_io(operand.clone()).await;
            }
            service.teardown(operand.clone()).await;
        }));
    }

    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_net_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone,
    {
        let mut service = S::new(config.clone());
        let ctx = service.ctx();

        let operand = self.operand.clone();
        let shutdown = self.shutdown.0.subscribe();

        if let Some(address) = ctx.address.clone() {
            log::debug!("Starting '{}' service on {}", ctx.name, address);
        } else {
            log::debug!("Starting '{}' service", ctx.name);
        }

        self.tasks.push(tokio::spawn(async move {
            service.setup(operand.clone()).await;
            while shutdown.is_empty() {
                service.wait_io(operand.clone()).await;
            }
            service.teardown(operand.clone()).await;
        }));
    }

    pub fn schedule_net2_service<S, C>(&mut self, config: C, duration: std::time::Duration)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone,
    {
        let mut interval = tokio::time::interval(duration);

        let mut service = S::new(config.clone());
        let ctx = service.ctx();

        let operand = self.operand.clone();
        let shutdown = self.shutdown.0.subscribe();

        if let Some(address) = ctx.address.clone() {
            log::debug!("Starting '{}' service on {}", ctx.name, address);
        } else {
            log::debug!("Starting '{}' service", ctx.name);
        }

        self.tasks.push(tokio::spawn(async move {
            service.setup(operand.clone()).await;
            while shutdown.is_empty() {
                interval.tick().await;
                service.tick(operand.clone()).await;
            }
            service.teardown(operand.clone()).await;
        }));
    }

    pub fn schedule_net3_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone,
    {
        let mut service = S::new(config.clone());
        let ctx = service.ctx();

        let operand = self.operand.clone();
        let _shutdown = self.shutdown.0.subscribe();
        let mut motion_rx = self.motion_rx.take().unwrap();

        if let Some(address) = ctx.address.clone() {
            log::debug!("Starting '{}' service on {}", ctx.name, address);
        } else {
            log::debug!("Starting '{}' service", ctx.name);
        }

        // TODO: Break from task on shutdown
        tokio::spawn(async move {
            while let Some(motion) = motion_rx.recv().await {
                // TODO: This only works when the queue is not blocking
                // if !shutdown.is_empty() {
                //     break;
                // }

                service.on_event(operand.clone(), &motion).await;
            }
        });
    }

    /// Schedule a component to run in the background.
    ///
    /// This method will schedule a component to run in the background. On each tick, the component
    /// will be provided with a component context and a mutable reference to the runtime state.
    pub fn schedule_service<S, C>(&mut self, config: C, duration: std::time::Duration)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone,
    {
        let mut interval = tokio::time::interval(duration);

        let mut service = S::new(config.clone());
        let ctx = service.ctx();

        let operand = self.operand.clone();
        let shutdown = self.shutdown.0.subscribe();

        if let Some(address) = ctx.address.clone() {
            log::debug!("Starting '{}' service on {}", ctx.name, address);
        } else {
            log::debug!("Starting '{}' service", ctx.name);
        }

        self.tasks.push(tokio::spawn(async move {
            service.setup(operand.clone()).await;
            while shutdown.is_empty() {
                interval.tick().await;
                service.tick(operand.clone()).await;
            }
            service.teardown(operand.clone()).await;
        }));
    }

    /// Schedule a component to run in the background with default configuration.
    ///
    /// This method will schedule a component to run in the background. On each tick, the component
    /// will be provided with a component context and a mutable reference to the runtime state.
    pub fn schedule_service_default<S>(&mut self, duration: std::time::Duration)
    where
        S: Service<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        let mut interval = tokio::time::interval(duration);

        let mut service = S::new(crate::runtime::NullConfig);
        let ctx = service.ctx();

        let operand = self.operand.clone();
        let shutdown = self.shutdown.0.subscribe();

        if let Some(address) = ctx.address.clone() {
            log::debug!("Starting '{}' service on {}", ctx.name, address);
        } else {
            log::debug!("Starting '{}' service", ctx.name);
        }

        self.tasks.push(tokio::spawn(async move {
            service.setup(operand.clone()).await;
            while shutdown.is_empty() {
                interval.tick().await;
                service.tick(operand.clone()).await;
            }
            service.teardown(operand.clone()).await;
        }));
    }

    pub async fn run_interval<S>(&mut self, mut service: S, duration: std::time::Duration)
    where
        S: Service<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        let mut interval = tokio::time::interval(duration);

        let ctx = service.ctx();

        let operand = self.operand.clone();
        let shutdown = self.shutdown.0.subscribe();
        // let motion_tx = self.motion_tx.clone();

        if let Some(address) = ctx.address.clone() {
            log::debug!("Starting '{}' service on {}", ctx.name, address);
        } else {
            log::debug!("Starting '{}' service", ctx.name);
        }

        service.setup(operand.clone()).await;
        while shutdown.is_empty() {
            interval.tick().await;
            service.tick(operand.clone()).await;
        }
        service.teardown(operand.clone()).await;
    }

    /// Enqueue a motion command.
    ///
    /// This method will enqueue a motion command to be sent to the network service.
    #[inline]
    pub async fn enqueue_motion(&self, motion: crate::core::Motion) {
        self.motion_tx.send(motion).await.ok();
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
