use crate::{config::Configurable, Runtime};

/// Runtime builder.
///
/// The runtime builder is a convenient wrapper around the runtime core. It
/// creates and configures the core based on the global config and current
/// environment. It then presents the caller with a simple method to launch
/// the runtime loop.
///
/// The runtime builder *must* be used to construct a runtime.
pub struct Builder<Cnf>(Runtime<Cnf>);

impl<Cnf: Configurable> Builder<Cnf> {
    /// Construct runtime service from configuration.
    ///
    /// Note that this method is certain to block.
    pub fn from_config(config: &Cnf) -> super::Result<Self> {
        use tokio::sync::broadcast;

        let (motion_tx, motion_rx) = tokio::sync::mpsc::channel(crate::consts::QUEUE_SIZE_MOTION);

        Ok(Self(Runtime::<Cnf> {
            config: config.clone(),
            instance: crate::core::Instance::default(),
            operand: crate::runtime::SharedOperandState::default(),
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

    /// Enqueue a motion command.
    ///
    /// This method will enqueue a motion command to be processed by the runtime.
    pub fn enqueue_startup_motion(self, motion: crate::core::Motion) -> Self {
        self.0.motion_tx.blocking_send(motion).unwrap();
        self
    }

    /// Build the runtime.
    #[inline]
    pub fn build(self) -> Runtime<Cnf> {
        self.0
    }
}
