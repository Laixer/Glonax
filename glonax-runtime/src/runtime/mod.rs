// mod component;
mod error;

use std::future::Future;

pub use self::error::Error;
// pub use component::{Component, ComponentContext, InitComponent, PostComponent};

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type CommandSender = tokio::sync::broadcast::Sender<crate::core::Object>;
pub type CommandReceiver = tokio::sync::broadcast::Receiver<crate::core::Object>;
pub type SignalSender = tokio::sync::broadcast::Sender<crate::core::Object>;
pub type SignalReceiver = tokio::sync::broadcast::Receiver<crate::core::Object>;

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
    fn setup(&mut self) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Teardown the service.
    ///
    /// This method is called once on shutdown and should be used to cleanup the service.
    fn teardown(&mut self) -> impl std::future::Future<Output = ()> + Send {
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
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn wait_io_pub(
        &mut self,
        _signal_tx: SignalSender,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Tick the component on interval.
    ///
    /// This method is called in conjunction with other services
    /// and should therefore be non-blocking. The method is optional
    /// and does not need to be implemented.
    fn tick(&mut self) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn command(
        &mut self,
        _object: &crate::core::Object,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }
}

pub trait NetworkService {
    fn setup2(&mut self) -> impl Future<Output = ()> + Send {
        async {}
    }

    fn teardown2(&mut self) -> impl Future<Output = ()> + Send {
        async {}
    }

    fn recv(&mut self, signal_tx: SignalSender) -> impl Future<Output = ()> + Send;

    fn on_tick(&mut self) -> impl Future<Output = ()> + Send;

    fn on_command(&mut self, object: &crate::core::Object) -> impl Future<Output = ()> + Send;
}

pub struct Runtime {
    /// Command sender.
    command_tx: CommandSender,
    /// Command receiver.
    command_rx: CommandReceiver,

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

impl std::default::Default for Runtime {
    fn default() -> Self {
        let (command_tx, command_rx) =
            tokio::sync::broadcast::channel(crate::consts::QUEUE_SIZE_COMMAND);
        let (signal_tx, signal_rx) =
            tokio::sync::broadcast::channel(crate::consts::QUEUE_SIZE_SIGNAL);

        Self {
            command_tx,
            command_rx,
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

        debug!("Enable shutdown signal");

        let sender = self.shutdown.0.clone();

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
    }

    /// Spawns a future onto the runtime's executor.
    ///
    /// This method spawns a future onto the runtime's executor, allowing it to run in the background.
    /// The future must implement the `Future` trait with an output type of `()`, and it must also be `Send` and `'static`.
    fn spawn<F: Future<Output = ()> + Send + 'static>(&mut self, f: F) {
        self.task_pool.push(tokio::spawn(f));
    }

    // TODO: Only the TCP Server uses `command_tx`
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

        log::debug!("Schedule IO service: {}", service.ctx());

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

        log::debug!("Schedule IO service: {}", service.ctx());

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

    pub fn schedule_net_service<S, C>(&mut self, config: C, duration: std::time::Duration)
    where
        S: Service<C> + Clone + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let mut command_rx = self.command_tx.subscribe();

        let signal_tx = self.signal_tx.clone();

        let mut service = S::new(config.clone());
        let mut service2 = service.clone();
        let mut service3 = service.clone();

        if self.shutdown.1.is_empty() {
            let mut shutdown = self.shutdown.0.subscribe();

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

            let mut shutdown = self.shutdown.0.subscribe();

            self.spawn(async move {
                tokio::select! {
                    _ = async {
                        loop {
                            service2.tick().await;
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
                        while let Ok(object) = command_rx.recv().await {
                            service3.command(&object).await;
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
