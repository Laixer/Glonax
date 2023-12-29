use crate::{config::Configurable, RobotState, Runtime};

/// Runtime builder.
///
/// The runtime builder is a convenient wrapper around the runtime core. It
/// creates and configures the core based on the global config and current
/// environment. It then presents the caller with a simple method to launch
/// the runtime loop.
///
/// The runtime builder *must* be used to construct a runtime.
pub struct Builder<Cnf, R>(Runtime<Cnf, R>);

impl<Cnf: Configurable, R: RobotState> Builder<Cnf, R> {
    /// Construct runtime service from configuration and instance.
    ///
    /// Note that this method is certain to block.
    pub fn new(config: &Cnf, instance: crate::core::Instance) -> super::Result<Self> {
        use tokio::sync::broadcast;

        let (motion_tx, motion_rx) = tokio::sync::mpsc::channel(crate::consts::QUEUE_SIZE_MOTION);

        Ok(Self(Runtime::<Cnf, R> {
            config: config.clone(),
            instance: instance.clone(),
            operand: std::sync::Arc::new(tokio::sync::RwLock::new(crate::Operand {
                instance,
                ..Default::default()
            })),
            motion_tx,
            motion_rx: Some(motion_rx),
            shutdown: broadcast::channel(1),
        }))
    }

    /// Listen for termination signal.
    ///
    /// This method will spawn a task that will listen for a termination signal.
    /// When the signal is received, the runtime will be shutdown.
    pub fn with_shutdown(self) -> Self {
        debug!("Enable shutdown signal");

        let sender = self.0.shutdown.0.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();

            info!("Termination requested");

            sender.send(()).unwrap();
        });

        self
    }

    /// Enqueue a motion command to be processed by the runtime on startup.
    ///
    /// This method will enqueue a motion command to be processed by the runtime.
    /// The motion command will be processed in the order it was received.
    pub fn enqueue_startup_motion(self, motion: crate::core::Motion) -> Self {
        self.0.motion_tx.try_send(motion).unwrap();
        self
    }

    /// Build the runtime.
    #[inline]
    pub fn build(self) -> Runtime<Cnf, R> {
        self.0
    }
}
