mod error;

use crate::{Configurable, RobotState};

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type MotionSender = tokio::sync::mpsc::Sender<crate::core::Motion>;
pub type MotionReceiver = tokio::sync::mpsc::Receiver<crate::core::Motion>;
pub type SharedOperandState<R> = std::sync::Arc<tokio::sync::RwLock<crate::Operand<R>>>;

pub mod builder;

// TODO: Move
// TODO: Send should be optional
pub trait Service<R>: Default + Send {
    fn run(&mut self, state: &mut R);
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
    /// Glonax operand.
    pub operand: SharedOperandState<R>, // TODO: Generic
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

    pub fn schedule_interval_service<S>(&self, duration: std::time::Duration)
    where
        S: Service<R> + 'static,
        R: RobotState + Send + Sync + 'static,
    {
        let mut interval = tokio::time::interval(duration);

        // TODO: Replace with some `new` method accepting a reference to the configuation
        let mut service = S::default();
        let opr = self.operand.clone();

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                let q = opr.clone();

                service.run(&mut q.write().await.state);
            }
        });
    }

    pub async fn run_interval_service<S>(&self, duration: std::time::Duration)
    where
        S: Service<R> + 'static,
        R: RobotState + 'static,
    {
        let mut interval = tokio::time::interval(duration);

        // TODO: Replace with some `new` method accepting a reference to the configuation
        let mut service = S::default();
        let opr = self.operand.clone();

        loop {
            interval.tick().await;

            let q = opr.clone();

            service.run(&mut q.write().await.state);
        }
    }

    /// Run a motion service.
    pub async fn run_motion_service<Fut>(
        &self,
        service: impl FnOnce(
            Cnf,
            SharedOperandState<R>,
            MotionSender,
            tokio::sync::broadcast::Receiver<()>,
        ) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        service(
            self.config.clone(),
            self.operand.clone(),
            self.motion_tx.clone(),
            self.shutdown_signal(),
        )
        .await;
    }

    // TODO: Rename to `spawn_control_service`
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

    /// Spawn a middleware service in the background.
    pub fn spawn_middleware_service<Fut>(
        &self,
        service: impl FnOnce(Cnf, SharedOperandState<R>) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(self.config.clone(), self.operand.clone()));
    }

    /// Wait for the runtime to shutdown.
    ///
    /// This method will block until the runtime is shutdown.
    pub async fn wait_for_shutdown(&self) {
        self.shutdown_signal().recv().await.ok();
    }
}
