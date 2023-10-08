pub(super) mod operand;

mod error;

use crate::Configurable;

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type SignalSender = tokio::sync::mpsc::Sender<crate::core::Signal>;
pub type SignalReceiver = tokio::sync::mpsc::Receiver<crate::core::Signal>;
pub type MotionSender = tokio::sync::mpsc::Sender<crate::core::Motion>;
pub type MotionReceiver = tokio::sync::mpsc::Receiver<crate::core::Motion>;
pub type SharedMachineState = std::sync::Arc<tokio::sync::RwLock<crate::MachineState>>;

pub mod builder;

pub struct RuntimeContext<Conf> {
    pub config: Conf,
    /// Glonax instance.
    pub instance: crate::core::Instance,
    pub signal_tx: SignalSender,
    pub signal_rx: Option<SignalReceiver>,
    pub motion_tx: MotionSender,
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

    /// Spawn an asynchronous task in the background.
    ///
    /// The task will be terminated when the runtime is shutdown or when the
    /// shutdown signal is received.
    pub fn spawn_background_task<T>(&self, task: T)
    where
        T: std::future::Future<Output = ()> + Send + 'static,
    {
        let mut shutdown = self.shutdown_signal();

        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown.recv() => {
                    log::debug!("Shutting down background task");
                }
                _ = task => {}
            }
        });
    }

    pub fn spawn_signal_service<Fut>(&self, service: impl FnOnce(Cnf, SignalSender) -> Fut)
    where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let local_config = self.config.clone();
        let local_sender = self.signal_tx.clone();

        tokio::spawn(service(local_config, local_sender));
    }

    pub async fn run_motion_service<Fut>(
        &self,
        machine_state: &SharedMachineState,
        service: impl FnOnce(
            Cnf,
            SharedMachineState,
            MotionSender,
            tokio::sync::broadcast::Receiver<()>,
        ) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let local_config = self.config.clone();
        let local_machine_state = machine_state.clone();
        let local_sender = self.motion_tx.clone();

        let shutdown = self.shutdown_signal();

        service(local_config, local_machine_state, local_sender, shutdown).await;
    }

    pub fn spawn_motion_sink<Fut>(&mut self, service: impl FnOnce(Cnf, MotionReceiver) -> Fut)
    where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let local_config = self.config.clone();
        let local_receiver = self.motion_rx.take().unwrap();

        tokio::spawn(service(local_config, local_receiver));
    }

    pub fn spawn_middleware_service<Fut>(
        &self,
        machine_state: &SharedMachineState,
        service: impl FnOnce(Cnf, SharedMachineState) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let local_config = self.config.clone();
        let local_machine_state = machine_state.clone();

        tokio::spawn(service(local_config, local_machine_state));
    }

    pub fn spawn_middleware_signal_sink<Conf, Fut>(
        &mut self,
        config: &Conf,
        machine_state: &SharedMachineState,
        service: impl FnOnce(Conf, SharedMachineState, SignalReceiver) -> Fut,
    ) where
        Fut: std::future::Future<Output = ()> + Send + 'static,
        Conf: crate::Configurable,
    {
        let local_config = config.clone();
        let local_machine_state = machine_state.clone();
        let local_receiver = self.signal_rx.take().unwrap();

        tokio::spawn(service(local_config, local_machine_state, local_receiver));
    }

    /// Wait for the runtime to shutdown.
    ///
    /// This method will block until the runtime is shutdown.
    pub async fn wait_for_shutdown(&self) {
        let mut shutdown = self.shutdown_signal();

        shutdown.recv().await.unwrap();
    }
}
