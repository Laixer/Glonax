pub(super) mod operand;

mod error;

use crate::{Configurable, SharedRuntimeState};

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type MotionSender = tokio::sync::mpsc::Sender<crate::core::Motion>;
pub type MotionReceiver = tokio::sync::mpsc::Receiver<crate::core::Motion>;

pub mod builder;

pub struct RuntimeContext<Conf> {
    /// Runtime configuration.
    pub config: Conf,
    /// Glonax instance.
    pub instance: crate::core::Instance,
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

impl<Cnf: Configurable> RuntimeContext<Cnf> {
    /// Listen for shutdown signal.
    #[inline]
    pub fn shutdown_signal(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown.0.subscribe()
    }

    /// Spawn a service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn spawn_service<Fut>(
        &self,
        runtime_state: &SharedRuntimeState,
        service: impl FnOnce(Cnf, SharedRuntimeState) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(self.config.clone(), runtime_state.clone()));
    }

    /// Run a motion service.
    pub async fn run_motion_service<Fut>(
        &self,
        runtime_state: &SharedRuntimeState,
        service: impl FnOnce(
            Cnf,
            SharedRuntimeState,
            MotionSender,
            tokio::sync::broadcast::Receiver<()>,
        ) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        service(
            self.config.clone(),
            runtime_state.clone(),
            self.motion_tx.clone(),
            self.shutdown_signal(),
        )
        .await;
    }

    /// Spawn a motion sink in the background.
    pub fn spawn_motion_sink<Fut>(
        &mut self,
        runtime_state: &SharedRuntimeState,
        service: impl FnOnce(Cnf, SharedRuntimeState, MotionReceiver) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(
            self.config.clone(),
            runtime_state.clone(),
            self.motion_rx.take().unwrap(),
        ));
    }

    /// Spawn a middleware service in the background.
    pub fn spawn_middleware_service<Fut>(
        &self,
        runtime_state: &SharedRuntimeState,
        service: impl FnOnce(Cnf, SharedRuntimeState) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        tokio::spawn(service(self.config.clone(), runtime_state.clone()));
    }

    /// Wait for the runtime to shutdown.
    ///
    /// This method will block until the runtime is shutdown.
    pub async fn wait_for_shutdown(&self) {
        self.shutdown_signal().recv().await.ok();
    }
}
