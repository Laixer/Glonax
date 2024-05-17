use crate::Runtime;

/// Runtime builder.
///
/// The runtime builder is a convenient wrapper around the runtime core. It
/// creates and configures the core based on the global config and current
/// environment. It then presents the caller with a simple method to launch
/// the runtime loop.
///
/// The runtime builder *must* be used to construct a runtime.
pub struct Builder<Cnf>(Runtime<Cnf>);

impl<Cnf: Clone> Builder<Cnf> {
    /// Construct runtime service from configuration.
    ///
    /// Note that this method is certain to block.
    pub fn new(config: &Cnf) -> super::Result<Self> {
        let (signal_tx, signal_rx) = std::sync::mpsc::channel();
        let (motion_tx, motion_rx) = tokio::sync::mpsc::channel(crate::consts::QUEUE_SIZE_MOTION);

        Ok(Self(Runtime::<Cnf> {
            config: config.clone(),
            operand: std::sync::Arc::new(tokio::sync::RwLock::new(crate::Operand {
                state: crate::MachineState::default(),
                governor: crate::Governor::new(800, 2_100), // TODO: Remove hardcoded values, use config
            })),
            signal_tx,
            signal_rx: Some(signal_rx),
            motion_tx,
            motion_rx: Some(motion_rx),
            tasks: Vec::new(),
            shutdown: tokio::sync::broadcast::channel(1),
        }))
    }

    /// Listen for termination signal.
    ///
    /// This method will spawn a task that will listen for the interrupt signal
    /// (SIGINT) and the termination signal (SIGTERM). The runtime will be
    /// gracefully terminated when either signal is received.
    pub fn with_shutdown(self) -> Self {
        use tokio::signal::unix;

        debug!("Enable shutdown signal");

        let sender = self.0.shutdown.0.clone();

        tokio::spawn(async move {
            let sigint = tokio::signal::ctrl_c();

            let mut binding = unix::signal(unix::SignalKind::terminate()).unwrap();
            let sigterm = binding.recv();

            tokio::select! {
                _ = sigint => log::debug!("Received SIGINT"),
                _ = sigterm => log::debug!("Received SIGTERM"),
            }

            info!("Termination requested");

            sender.send(()).unwrap();
        });

        self
    }

    /// Build the runtime.
    #[inline]
    pub fn build(self) -> Runtime<Cnf> {
        self.0
    }
}
