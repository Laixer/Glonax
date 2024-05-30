mod component;
mod error;

pub use self::error::Error;
pub use component::{Component, ComponentContext, InitComponent, PostComponent};

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type IPCSender = std::sync::mpsc::Sender<crate::core::ObjectMessage>;
pub type IPCReceiver = std::sync::mpsc::Receiver<crate::core::ObjectMessage>;
pub type CommandSender = tokio::sync::mpsc::Sender<crate::core::Object>;
pub type CommandReceiver = tokio::sync::mpsc::Receiver<crate::core::Object>;
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
    fn wait_io(
        &mut self,
        _ipc_tx: IPCSender,
        _command_tx: CommandSender,
        _signal_rx: SignalReceiver,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    /// Tick the component on interval.
    ///
    /// This method is called in conjunction with other services
    /// and should therefore be non-blocking. The method is optional
    /// and does not need to be implemented.
    fn tick(
        &mut self,
        _ipc_rx: std::rc::Rc<IPCReceiver>,
        _command_tx: CommandSender,
        _signal_tx: std::rc::Rc<SignalSender>,
        _pre_tick: bool,
    ) {
    }

    fn on_command(
        &mut self,
        _object: &crate::core::Object,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }
}

pub trait Executor {
    /// Runs the initialization logic of the executor.
    ///
    /// # Arguments
    ///
    /// * `ipc_rx` - A reference-counted pointer to the IPC receiver.
    fn run_init(&mut self, ipc_rx: std::rc::Rc<IPCReceiver>);

    /// Runs the main tick logic of the executor.
    fn run_tick(&mut self);

    /// Runs the post-processing logic of the executor.
    ///
    /// # Arguments
    ///
    /// * `command_tx` - The command sender for sending commands.
    /// * `signal_tx` - A reference-counted pointer to the signal sender.
    fn run_post(&mut self, command_tx: CommandSender, signal_tx: std::rc::Rc<SignalSender>);
}

pub struct Runtime {
    /// IPC sender.
    ipc_tx: IPCSender,
    /// IPC receiver.
    ipc_rx: Option<IPCReceiver>,
    /// Command sender.
    command_tx: CommandSender,
    /// Command receiver.
    command_rx: Option<CommandReceiver>,

    signal_tx: Option<SignalSender>,
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
        let (ipc_tx, ipc_rx) = std::sync::mpsc::channel();
        let (command_tx, command_rx) =
            tokio::sync::mpsc::channel(crate::consts::QUEUE_SIZE_COMMAND);
        let (signal_tx, signal_rx) = tokio::sync::broadcast::channel(8);

        Self {
            ipc_tx,
            ipc_rx: Some(ipc_rx),
            command_tx,
            command_rx: Some(command_rx),
            signal_tx: Some(signal_tx),
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
    fn spawn<F: std::future::Future<Output = ()> + Send + 'static>(&mut self, f: F) {
        self.task_pool.push(tokio::spawn(f));
    }

    // TODO: Only the TCP Server uses `command_tx`
    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let command_tx = self.command_tx.clone();
        let ipc_tx = self.ipc_tx.clone();
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
                            service.wait_io(ipc_tx.clone(), command_tx.clone(), signal_rx.resubscribe()).await;
                        }
                    } => {}
                    _ = shutdown.recv() => {}
                }

                service.teardown().await;
            });
        }
    }

    // TODO: Only the TCP Server uses `command_tx`
    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_service2<S, C>(&mut self, config: C)
    where
        S: Service<C> + Clone + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let mut command_rx = self.command_rx.take().unwrap();
        let command_tx = self.command_tx.clone();
        let ipc_tx = self.ipc_tx.clone();
        let signal_rx = self.signal_rx.resubscribe();
        let mut shutdown = self.shutdown.0.subscribe();
        let mut shutdown2 = self.shutdown.0.subscribe();

        let mut service = S::new(config.clone());
        let mut service2 = service.clone();

        if self.shutdown.1.is_empty() {
            self.spawn(async move {
                service.setup().await;

                tokio::select! {
                    _ = async {
                        loop {
                            service.wait_io(ipc_tx.clone(), command_tx.clone(), signal_rx.resubscribe()).await;

                        }
                    } => {}
                    _ = shutdown.recv() => {}
                }

                service.teardown().await;
            });

            self.spawn(async move {
                tokio::select! {
                    _ = async {
                        while let Some(object) = command_rx.recv().await {
                            service2.on_command(&object).await;
                        }
                    } => {}
                    _ = shutdown2.recv() => {}
                }
            });
        }
    }

    /// Listen for command event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_command_service<S, C>(&mut self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: Clone + Send + 'static,
    {
        let mut command_rx = self.command_rx.take().unwrap();
        let mut shutdown = self.shutdown.0.subscribe();

        let mut service = S::new(config.clone());

        log::debug!("Schedule command service: {}", service.ctx());

        if self.shutdown.1.is_empty() {
            self.spawn(async move {
                tokio::select! {
                    _ = async {
                        while let Some(command) = command_rx.recv().await {
                            service.on_command(&command).await;
                        }
                    } => {}
                    _ = shutdown.recv() => {}
                }
            });
        }
    }

    /// Run a service in the background.
    ///
    /// This method will run a service in the background. The service will be provided with a copy of
    /// the runtime configuration and a reference to the runtime.
    pub async fn run_interval(
        &mut self,
        mut service: impl Executor,
        duration: std::time::Duration,
    ) {
        let ipc_rx = std::rc::Rc::new(self.ipc_rx.take().unwrap());
        let signal_tx = std::rc::Rc::new(self.signal_tx.take().unwrap());

        while self.shutdown.1.is_empty() {
            let tick_start = std::time::Instant::now();

            service.run_init(ipc_rx.clone());
            service.run_tick();

            let tick_duration = tick_start.elapsed();
            log::trace!("Tick loop duration: {:?}", tick_duration);

            if tick_duration > duration {
                log::warn!("Tick loop delta is too high: {:?}", tick_duration);
            } else {
                tokio::time::sleep(duration - tick_duration).await;
            }

            service.run_post(self.command_tx.clone(), signal_tx.clone());
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
