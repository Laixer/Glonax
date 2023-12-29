mod error;

use crate::{Configurable, RobotState};

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type MotionSender = tokio::sync::mpsc::Sender<crate::core::Motion>;
pub type MotionReceiver = tokio::sync::mpsc::Receiver<crate::core::Motion>;
pub type SharedOperandState<R> = std::sync::Arc<tokio::sync::RwLock<crate::Operand<R>>>;

pub mod builder;

pub trait Component<Cnf: Configurable, R: RobotState> {
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
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut R);
}

/// Component context.
///
/// The component context is provided to each component on each tick. All
/// data provided to the component is non-persistent and will be lost on
/// the next tick.
pub struct ComponentContext {
    /// Motion command sender.
    motion_tx: tokio::sync::mpsc::Sender<crate::core::Motion>,
    // TODO: Maybe target needs to be moved to state.
    /// Target position.
    pub target: Option<crate::core::Target>,
    /// Instance.
    instance: crate::core::Instance,
}

impl ComponentContext {
    pub fn new(
        motion_tx: tokio::sync::mpsc::Sender<crate::core::Motion>,
        instance: crate::core::Instance,
    ) -> Self {
        Self {
            motion_tx,
            target: None,
            instance,
        }
    }

    /// Commit a motion command.
    pub fn commit(&mut self, motion: crate::core::Motion) {
        if let Err(e) = self.motion_tx.try_send(motion) {
            log::error!("Failed to send motion command: {}", e);
        }
    }

    /// Retrieve the instance.
    #[inline]
    pub fn instance(&self) -> &crate::core::Instance {
        &self.instance
    }
}

/// Construct runtime service from configuration and instance.
///
/// Note that this method is certain to block.
pub fn builder<Cnf: Configurable, R: RobotState>(
    config: &Cnf,
    instance: crate::core::Instance,
) -> self::Result<builder::Builder<Cnf, R>> {
    builder::Builder::new(config, instance)
}

pub struct Runtime<Conf, R> {
    /// Runtime configuration.
    pub config: Conf,
    /// Instance.
    pub instance: crate::core::Instance,
    /// Glonax operand.
    pub operand: SharedOperandState<R>, // TODO: Generic, TODO: Remove instance from operand.
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

impl<Cnf: Configurable, R> Runtime<Cnf, R> {
    /// Listen for shutdown signal.
    #[inline]
    pub fn shutdown_signal(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown.0.subscribe()
    }

    /// Spawn a service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn spawn_service<Fut>(&self, service: impl FnOnce(Cnf, SharedOperandState<R>) -> Fut)
    where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(self.config.clone(), self.operand.clone()));
    }

    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_service<Fut>(&self, service: impl FnOnce(Cnf, SharedOperandState<R>) -> Fut)
    where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(self.config.clone(), self.operand.clone()));
    }

    pub fn schedule_interval<C>(&self, duration: std::time::Duration)
    where
        C: Component<Cnf, R> + Send + Sync + 'static,
        Cnf: Configurable,
        R: RobotState + Send + Sync + 'static,
    {
        let mut interval = tokio::time::interval(duration);

        let mut component = C::new(self.config.clone());

        let motion_tx = self.motion_tx.clone();
        let instance = self.instance.clone();
        let operand = self.operand.clone();

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                let mut ctx = ComponentContext::new(motion_tx.clone(), instance.clone());
                let mut runtime_state = operand.write().await;

                component.tick(&mut ctx, &mut runtime_state.state);
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
        C: Component<Cnf, R>,
        Cnf: Configurable,
        R: RobotState,
    {
        let mut interval = tokio::time::interval(duration);

        loop {
            interval.tick().await;

            let mut ctx = ComponentContext::new(self.motion_tx.clone(), self.instance.clone());
            let mut runtime_state = self.operand.write().await;

            component.tick(&mut ctx, &mut runtime_state.state);
        }
    }

    /// Run a motion service.
    pub fn spawn_motion_service<Fut>(
        &self,
        service: impl FnOnce(Cnf, SharedOperandState<R>, MotionSender) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(
            self.config.clone(),
            self.operand.clone(),
            self.motion_tx.clone(),
        ));
    }

    /// Spawn a motion sink in the background.
    pub fn spawn_motion_sink<Fut>(
        &mut self,
        service: impl FnOnce(Cnf, SharedOperandState<R>, MotionReceiver) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(
            self.config.clone(),
            self.operand.clone(),
            self.motion_rx.take().unwrap(),
        ));
    }

    /// Wait for the runtime to shutdown.
    ///
    /// This method will block until the runtime is shutdown.
    pub async fn wait_for_shutdown(&self) {
        self.shutdown_signal().recv().await.ok();
    }
}
