mod error;
mod j1939;

use std::{future::Future, time::Duration};

use tokio::sync::broadcast::{error::RecvError, Receiver, Sender};

pub use self::error::Error;
pub use self::j1939::{J1939Unit, J1939UnitError, NetDriverContext, NetworkService};

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type CommandSender = Sender<crate::core::Object>;
pub type CommandReceiver = Receiver<crate::core::Object>;
pub type SignalSender = Sender<crate::core::Object>;
pub type SignalReceiver = Receiver<crate::core::Object>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NullConfig;

pub struct ServiceContext {
    /// Service name.
    name: String,
    /// Service address.
    address: Option<String>,
}

impl ServiceContext {
    /// Construct a new service context.
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            address: None,
        }
    }

    /// Construct a new service context with address.
    pub fn with_address(name: impl ToString, address: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            address: Some(address.to_string()),
        }
    }
}

impl std::fmt::Display for ServiceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(address) = &self.address {
            write!(f, "{} on {}", self.name, address)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

pub trait Service<Cnf> {
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
    ///
    /// This method is called once on startup and should be used to initialize the service.
    fn setup(&mut self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Teardown the service.
    ///
    /// This method is called once on shutdown and should be used to cleanup the service.
    fn teardown(&mut self) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Wait for IO event.
    ///
    /// This method is always called on a separate thread and
    /// should be used to wait for IO events. The method is optional
    /// and does not need to be implemented.
    fn wait_io_sub(
        &mut self,
        _command_tx: CommandSender,
        _signal_rx: SignalReceiver,
    ) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn wait_io_pub(&mut self, _signal_tx: SignalSender) -> impl Future<Output = ()> + Send {
        std::future::ready(())
    }
}

pub struct Runtime {
    /// Command sender.
    command_tx: CommandSender,
    /// Signal sender.
    signal_tx: SignalSender,
    /// Signal receiver.
    signal_rx: SignalReceiver,
    /// Runtime tasks.
    task_pool: Vec<tokio::task::JoinHandle<()>>,
    /// Runtime event bus.
    shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
}

impl Default for Runtime {
    fn default() -> Self {
        let (command_tx, _) = tokio::sync::broadcast::channel(crate::consts::QUEUE_SIZE_COMMAND);
        let (signal_tx, signal_rx) =
            tokio::sync::broadcast::channel(crate::consts::QUEUE_SIZE_SIGNAL);

        Self {
            command_tx,
            signal_tx,
            signal_rx,
            task_pool: Vec::new(),
            shutdown: tokio::sync::broadcast::channel(1),
        }
    }
}

impl Runtime {
    /// Listen for termination signal.
    ///
    /// This method will spawn a task that will listen for the interrupt signal
    /// (SIGINT) and the termination signal (SIGTERM). The runtime will be
    /// gracefully terminated when either signal is received.
    pub fn register_shutdown_signal(&self) {
        use tokio::signal::unix;

        debug!("Register shutdown signal");

        let sender = self.shutdown.0.clone();

        tokio::spawn(async move {
            let sigint = tokio::signal::ctrl_c();

            let mut binding = unix::signal(unix::SignalKind::terminate()).unwrap();
            let sigterm = binding.recv();

            tokio::select! {
                _ = sigint => debug!("Received SIGINT"),
                _ = sigterm => debug!("Received SIGTERM"),
            }

            info!("Termination requested");

            sender.send(()).unwrap();
        });
    }

    /// Spawns a future onto the runtime's executor.
    ///
    /// This method spawns a future onto the runtime's executor, allowing it to run in the background.
    /// The future must implement the `Future` trait with an output type of `()`, and it must also be `Send` and `'static`.
    fn spawn<F: Future<Output = ()> + Send + 'static>(&mut self, f: F) {
        self.task_pool.push(tokio::spawn(f));
    }

    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_sub_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let command_tx = self.command_tx.clone();
        let signal_rx = self.signal_rx.resubscribe();
        let mut shutdown = self.shutdown.0.subscribe();

        let mut service = S::new(config.clone());

        debug!("Schedule IO service: {}", service.ctx());

        if self.shutdown.1.is_empty() {
            self.spawn(async move {
                service.setup().await;

                tokio::select! {
                    _ = async {
                        loop {
                            service.wait_io_sub(command_tx.clone(), signal_rx.resubscribe()).await;
                        }
                    } => {}
                    _ = shutdown.recv() => {}
                }

                service.teardown().await;
            });
        }
    }

    pub fn schedule_io_pub_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let signal_tx = self.signal_tx.clone();
        let mut shutdown = self.shutdown.0.subscribe();

        let mut service = S::new(config.clone());

        debug!("Schedule IO service: {}", service.ctx());

        if self.shutdown.1.is_empty() {
            self.spawn(async move {
                service.setup().await;

                tokio::select! {
                    _ = async {
                        loop {
                            service.wait_io_pub(signal_tx.clone()).await;
                        }
                    } => {}
                    _ = shutdown.recv() => {}
                }

                service.teardown().await;
            });
        }
    }

    pub fn schedule_net_service<S, C>(&mut self, config: C, duration: Duration)
    where
        S: NetworkService<C> + Clone + Send + 'static,
        C: Clone + Send + 'static,
    {
        let mut command_rx = self.command_tx.subscribe();

        let signal1_tx = self.signal_tx.clone();
        let signal2_tx = self.signal_tx.clone();

        let mut service1 = S::new(config.clone());
        let mut service2 = service1.clone();
        let mut service3 = service1.clone();

        if self.shutdown.1.is_empty() {
            let mut shutdown = self.shutdown.0.subscribe();

            self.spawn(async move {
                service1.setup().await;

                tokio::select! {
                    _ = async {
                        loop {
                            service1.recv(signal1_tx.clone()).await;
                        }
                    } => {}
                    _ = shutdown.recv() => {}
                }

                service1.teardown().await;
            });

            let mut shutdown = self.shutdown.0.subscribe();

            self.spawn(async move {
                tokio::select! {
                    _ = async {
                        loop {
                            service2.on_tick(signal2_tx.clone()).await;
                            tokio::time::sleep(duration).await;
                        }
                    } => {}
                    _ = shutdown.recv() => {}
                }
            });

            let mut shutdown = self.shutdown.0.subscribe();

            self.spawn(async move {
                tokio::select! {
                    _ = async {
                        loop {
                            match command_rx.recv().await {
                                Ok(object) => {
                                    service3.on_command(&object).await;
                                }
                                Err(RecvError::Lagged(count)) => {
                                    warn!("Command receiver lagged by {} objects", count);
                                }
                                Err(RecvError::Closed) => {
                                    break;
                                }
                            }
                        }
                    } => {}
                    _ = shutdown.recv() => {}
                }
            });
        }
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
        for task in self.task_pool.drain(..) {
            task.await.ok();
        }
    }
}
