mod error;

use crate::{core::Instance, world::World, Configurable, MachineState};

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type MotionSender = tokio::sync::mpsc::Sender<crate::core::Motion>;
pub type MotionReceiver = tokio::sync::mpsc::Receiver<crate::core::Motion>;
pub type SharedOperandState = std::sync::Arc<tokio::sync::RwLock<crate::Operand>>;

pub mod builder;

pub trait Component<Cnf: Configurable> {
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
    world: World,
    /// Actuator values.
    actuators: std::collections::HashMap<u16, f32>,
    /// Last tick.
    last_tick: std::time::Instant,
}

impl ComponentContext {
    /// Construct a new component context.
    pub fn new(motion_tx: tokio::sync::mpsc::Sender<crate::core::Motion>) -> Self {
        Self {
            motion_tx,
            world: World::default(),
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

    /// Retrieve the world mutably.
    #[inline]
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Retrieve the world.
    #[inline]
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Insert a value into the context.
    #[inline]
    pub fn map(&mut self, key: u16, value: f32) {
        self.actuators.insert(key, value);
    }

    /// Retrieve a value from the context.
    #[inline]
    pub fn get(&self, key: u16) -> Option<&f32> {
        self.actuators.get(&key)
    }

    /// Retrieve the tick delta.
    pub fn delta(&self) -> std::time::Duration {
        self.last_tick.elapsed()
    }
}

/// Construct runtime service from configuration and instance.
///
/// Note that this method is certain to block.
pub fn builder<Cnf: Configurable>(
    config: &Cnf,
    instance: crate::core::Instance,
) -> self::Result<builder::Builder<Cnf>> {
    builder::Builder::new(config, instance)
}

pub struct Runtime<Conf> {
    /// Runtime configuration.
    pub config: Conf,
    /// Instance.
    pub instance: crate::core::Instance,
    /// Glonax operand.
    pub operand: SharedOperandState, // TODO: Generic, TODO: Remove instance from operand.
    /// Motion command sender.
    pub motion_tx: MotionSender,
    /// Motion command receiver.
    pub motion_rx: Option<MotionReceiver>,
    /// Runtime event bus.
    pub shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
}

impl<Cnf: Configurable> Runtime<Cnf> {
    /// Listen for shutdown signal.
    #[inline]
    pub fn shutdown_signal(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown.0.subscribe()
    }

    /// Spawn a service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn spawn_service<Fut>(&self, service: impl FnOnce(Cnf, SharedOperandState) -> Fut)
    where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(self.config.clone(), self.operand.clone()));
    }

    // TODO: Add instance to new
    /// Create a dynamic component with the given order.
    ///
    /// This method will create a dynamic component with the given order. The component will be
    /// provided with a copy of the runtime configuration.
    pub fn make_dynamic<C>(&self, order: i32) -> (i32, Box<dyn Component<Cnf>>)
    where
        C: Component<Cnf> + Send + Sync + 'static,
        Cnf: Configurable,
    {
        (order, Box::new(C::new(self.config.clone())))
    }

    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_service<Fut>(
        &self,
        service: impl FnOnce(Cnf, Instance, SharedOperandState, MotionSender) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(
            self.config.clone(),
            self.instance.clone(),
            self.operand.clone(),
            self.motion_tx.clone(),
        ));
    }

    /// Spawn a motion sink in the background.
    pub fn schedule_motion_sink<Fut>(
        &mut self,
        service: impl FnOnce(Cnf, Instance, SharedOperandState, MotionReceiver) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(
            self.config.clone(),
            self.instance.clone(),
            self.operand.clone(),
            self.motion_rx.take().unwrap(),
        ));
    }

    /// Schedule a component to run in the background.
    ///
    /// This method will schedule a component to run in the background. On each tick, the component
    /// will be provided with a component context and a mutable reference to the runtime state.
    pub fn schedule_interval<C>(&self, duration: std::time::Duration)
    where
        C: Component<Cnf> + Send + Sync + 'static,
        Cnf: Configurable,
    {
        let mut interval = tokio::time::interval(duration);

        let mut component = C::new(self.config.clone());

        let operand = self.operand.clone();

        let mut ctx = ComponentContext::new(self.motion_tx.clone());
        tokio::spawn(async move {
            component.once(&mut ctx, &mut operand.write().await.state);

            ctx.last_tick = std::time::Instant::now();

            loop {
                interval.tick().await;

                let mut runtime_state = operand.write().await;

                component.tick(&mut ctx, &mut runtime_state.state);

                ctx.last_tick = std::time::Instant::now();
            }
        });
    }

    /// Run a component in the main thread.
    ///
    /// This method will run a component in the main thread until the runtime is shutdown.
    /// On each tick, the component will be provided with a component context and a mutable
    /// reference to the runtime state.
    pub async fn run_interval<C>(&self, mut component: C, duration: std::time::Duration)
    where
        C: Component<Cnf>,
        Cnf: Configurable,
    {
        let mut interval = tokio::time::interval(duration);

        let mut ctx = ComponentContext::new(self.motion_tx.clone());

        component.once(&mut ctx, &mut self.operand.write().await.state);

        ctx.last_tick = std::time::Instant::now();

        loop {
            interval.tick().await;

            let mut runtime_state = self.operand.write().await;

            component.tick(&mut ctx, &mut runtime_state.state);

            ctx.last_tick = std::time::Instant::now();
        }
    }

    /// Wait for the runtime to shutdown.
    ///
    /// This method will block until the runtime is shutdown.
    pub async fn wait_for_shutdown(&self) {
        self.shutdown_signal().recv().await.ok();
    }
}
