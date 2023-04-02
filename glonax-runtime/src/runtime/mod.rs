pub(super) mod operand;

mod error;

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub mod builder;

pub struct RuntimeContext {
    /// Runtime event bus.
    pub shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
}

impl RuntimeContext {
    /// Listen for shutdown signal.
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

    /// Wait for the runtime to shutdown.
    /// 
    /// This method will block until the runtime is shutdown.
    pub async fn wait_for_shutdown(&self) {
        let mut shutdown = self.shutdown_signal();

        shutdown.recv().await.unwrap();
    }
}
