mod error;

use crate::{
    core::{Instance, Target},
    driver::net::NetDriver,
    world::World,
    Configurable, MachineState,
};

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type MotionSender = tokio::sync::mpsc::Sender<crate::core::Motion>;
pub type MotionReceiver = tokio::sync::mpsc::Receiver<crate::core::Motion>;
pub type SharedOperandState = std::sync::Arc<tokio::sync::RwLock<crate::Operand>>;

pub mod builder;

pub trait Component<Cnf: Configurable> {
    // TODO: Add instance to new
    /// Construct a new component.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    /// Run the component once.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn once(&mut self, _ctx: &mut ComponentContext, _state: &mut MachineState) {}

    /// Tick the component.
    ///
    /// This method will be called on each tick of the runtime.
    /// How often the runtime ticks is determined by the runtime configuration.
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState);
}

/// Component context.
///
/// The component context is provided to each component on each tick. The
/// component context is used to communicate within the component pipeline.
pub struct ComponentContext {
    /// Motion command sender.
    motion_tx: tokio::sync::mpsc::Sender<crate::core::Motion>,
    /// World.
    pub world: World,
    /// Current target.
    pub target: Option<Target>,
    /// Actuator values.
    pub actuators: std::collections::HashMap<u16, f32>, // TODO: Find another way to pass actuator errors around.
    /// Last tick.
    last_tick: std::time::Instant,
}

impl ComponentContext {
    /// Construct a new component context.
    pub fn new(motion_tx: tokio::sync::mpsc::Sender<crate::core::Motion>) -> Self {
        Self {
            motion_tx,
            world: World::default(),
            target: None,
            actuators: std::collections::HashMap::new(),
            last_tick: std::time::Instant::now(),
        }
    }

    /// Commit a motion command.
    pub fn commit(&mut self, motion: crate::core::Motion) {
        if let Err(e) = self.motion_tx.try_send(motion) {
            log::error!("Failed to send motion command: {}", e);
        }
    }

    /// Retrieve the tick delta.
    pub fn delta(&self) -> std::time::Duration {
        self.last_tick.elapsed()
    }

    /// Called after all components are ticked.
    fn post_tick(&mut self) {
        self.actuators.clear();
        self.last_tick = std::time::Instant::now();
    }
}

/// Construct runtime service from configuration and instance.
///
/// Note that this method is certain to block.
#[inline]
pub fn builder<Cnf: Configurable>(
    config: &Cnf,
    instance: crate::core::Instance,
) -> self::Result<builder::Builder<Cnf>> {
    builder::Builder::new(config, instance)
}

pub struct Runtime<Conf> {
    /// Runtime configuration.
    pub config: Conf,
    /// Instance.
    pub instance: crate::core::Instance,
    /// Glonax operand.
    pub operand: SharedOperandState, // TODO: Generic, TODO: Remove instance from operand.
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

impl<Cnf: Configurable + Send + 'static> Runtime<Cnf> {
    /// Listen for shutdown signal.
    #[inline]
    pub fn shutdown_signal(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown.0.subscribe()
    }

    // TODO: Add instance to new
    /// Create a dynamic component with the given order.
    ///
    /// This method will create a dynamic component with the given order. The component will be
    /// provided with a copy of the runtime configuration.
    pub fn make_dynamic<C>(&self, order: i32) -> (i32, Box<dyn Component<Cnf>>)
    where
        C: Component<Cnf> + Send + Sync + 'static,
        Cnf: Configurable,
    {
        (order, Box::new(C::new(self.config.clone())))
    }

    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_service<Fut>(
        &self,
        service: impl FnOnce(Cnf, Instance, SharedOperandState, MotionSender) -> Fut + Send + 'static,
    ) where
        Fut: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
    {
        let config = self.config.clone();
        let instance = self.instance.clone();
        let operand = self.operand.clone();
        let motion_tx = self.motion_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = service(config, instance, operand, motion_tx).await {
                log::error!("Failed to start IO service: {}", e);
            }
        });
    }

    // /// Listen for network service in the background.
    // ///
    // /// This method will spawn a service in the background and return immediately. The service
    // /// will be provided with a copy of the operand and the interface name.
    // pub fn schedule_j1939_service<Fut>(
    //     &self,
    //     service: impl FnOnce(String, SharedOperandState, tokio::sync::broadcast::Receiver<()>) -> Fut
    //         + Send
    //         + 'static,
    //     interface: &str,
    // ) where
    //     Fut: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
    // {
    //     let operand = self.operand.clone();
    //     let interface = interface.to_owned();
    //     let shutdown = self.shutdown.0.subscribe();

    //     tokio::spawn(async move {
    //         if let Err(e) = service(interface, operand, shutdown).await {
    //             log::error!("Failed to start network service: {}", e);
    //         }
    //     });
    // }

    pub fn schedule_j1939_service_rx(&self, network: Vec<NetDriver>, interface: &str) {
        let operand = self.operand.clone();
        let interface = interface.to_owned();
        let shutdown = self.shutdown.0.subscribe();

        tokio::spawn(async move {
            if let Err(e) = rx_network(network, interface, operand, shutdown).await {
                log::error!("Failed to start network service: {}", e);
            }
        });
    }
    pub fn schedule_j1939_service_tx(&self, network: Vec<NetDriver>, interface: &str) {
        let operand = self.operand.clone();
        let interface = interface.to_owned();
        let shutdown = self.shutdown.0.subscribe();

        tokio::spawn(async move {
            if let Err(e) = tx_network(network, interface, operand, shutdown).await {
                log::error!("Failed to start network service: {}", e);
            }
        });
    }

    /// Listen for internal signals to trigger the service.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the operand and the interface name.
    pub fn schedule_j1939_motion_service<Fut>(
        &mut self,
        service: impl FnOnce(String, SharedOperandState, MotionReceiver) -> Fut + Send + 'static,
        interface: &str,
    ) where
        Fut: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
    {
        let operand = self.operand.clone();
        let interface = interface.to_owned();
        let motion_rx = self.motion_rx.take().unwrap();

        tokio::spawn(async move {
            if let Err(e) = service(interface, operand, motion_rx).await {
                log::error!("Failed to start network service: {}", e);
            }
        });
    }

    /// Spawn a motion sink in the background.
    pub fn schedule_motion_sink<Fut>(
        &mut self,
        service: impl FnOnce(Cnf, Instance, SharedOperandState, MotionReceiver) -> Fut + Send + 'static,
    ) where
        Fut: std::future::Future<Output = std::io::Result<()>> + Send + 'static,
    {
        let config = self.config.clone();
        let instance = self.instance.clone();
        let operand = self.operand.clone();
        let motion_rx = self.motion_rx.take().unwrap();

        tokio::spawn(async move {
            if let Err(e) = service(config, instance, operand, motion_rx).await {
                log::error!("Failed to start motion service: {}", e);
            }
        });
    }

    /// Schedule a component to run in the background.
    ///
    /// This method will schedule a component to run in the background. On each tick, the component
    /// will be provided with a component context and a mutable reference to the runtime state.
    pub fn schedule_interval<C>(&self, duration: std::time::Duration)
    where
        C: Component<Cnf> + Send + Sync + 'static,
        Cnf: Configurable,
    {
        let mut interval = tokio::time::interval(duration);

        let mut component = C::new(self.config.clone());

        let operand = self.operand.clone();

        let mut ctx = ComponentContext::new(self.motion_tx.clone());
        tokio::spawn(async move {
            component.once(&mut ctx, &mut operand.write().await.state);
            ctx.post_tick();

            loop {
                interval.tick().await;

                component.tick(&mut ctx, &mut operand.write().await.state);
                ctx.post_tick();
            }
        });
    }

    // TODO: Maybe copy MachineState to component state on each tick?
    /// Run a component in the main thread.
    ///
    /// This method will run a component in the main thread until the runtime is shutdown.
    /// On each tick, the component will be provided with a component context and a mutable
    /// reference to the runtime state.
    pub async fn run_interval<C>(&self, mut component: C, duration: std::time::Duration)
    where
        C: Component<Cnf>,
        Cnf: Configurable,
    {
        let mut interval = tokio::time::interval(duration);

        let mut ctx = ComponentContext::new(self.motion_tx.clone());

        component.once(&mut ctx, &mut self.operand.write().await.state);
        ctx.post_tick();

        while self.shutdown.1.is_empty() {
            interval.tick().await;

            component.tick(&mut ctx, &mut self.operand.write().await.state);
            ctx.post_tick();
        }
    }

    /// Enqueue a motion command.
    #[inline]
    pub async fn enqueue_motion(&self, motion: crate::core::Motion) {
        self.motion_tx.send(motion).await.ok();
    }

    /// Wait for the runtime to shutdown.
    ///
    /// This method will block until the runtime is shutdown.
    pub async fn wait_for_shutdown(&mut self) {
        self.shutdown.1.recv().await.ok();
    }
}

// TODO: Turn into service
async fn rx_network(
    mut network: Vec<NetDriver>,
    interface: String,
    runtime_state: SharedOperandState,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> std::io::Result<()> {
    use crate::driver::net::J1939Unit;

    /// J1939 name manufacturer code.
    const J1939_NAME_MANUFACTURER_CODE: u16 = 0x717;
    /// J1939 name function instance.
    const J1939_NAME_FUNCTION_INSTANCE: u8 = 6;
    /// J1939 name ECU instance.
    const J1939_NAME_ECU_INSTANCE: u8 = 0;
    /// J1939 name function.
    const J1939_NAME_FUNCTION: u8 = 0x1C;
    /// J1939 name vehicle system.
    const J1939_NAME_VEHICLE_SYSTEM: u8 = 2;

    log::debug!("Starting J1939 service on {}", interface);

    let socket = crate::net::CANSocket::bind(&crate::net::SockAddrCAN::new(&interface))?;
    let mut router = crate::net::Router::new(socket);

    let name = j1939::NameBuilder::default()
        .identity_number(0x1)
        .manufacturer_code(J1939_NAME_MANUFACTURER_CODE)
        .function_instance(J1939_NAME_FUNCTION_INSTANCE)
        .ecu_instance(J1939_NAME_ECU_INSTANCE)
        .function(J1939_NAME_FUNCTION)
        .vehicle_system(J1939_NAME_VEHICLE_SYSTEM)
        .build();

    router
        .inner()
        .send(&j1939::protocol::address_claimed(0x27, name))
        .await?;

    while shutdown.is_empty() {
        if let Err(e) = router.listen().await {
            log::error!("Failed to receive from router: {}", e);
        }

        for driver in network.iter_mut() {
            match driver {
                NetDriver::KueblerEncoder(enc) => {
                    enc.try_accept(&router, runtime_state.clone()).await;
                }
                NetDriver::KueblerInclinometer(imu) => {
                    imu.try_accept(&router, runtime_state.clone()).await;
                }
                NetDriver::EngineManagementSystem(ems) => {
                    ems.try_accept(&router, runtime_state.clone()).await;
                }
                NetDriver::HydraulicControlUnit(hcu) => {
                    hcu.try_accept(&router, runtime_state.clone()).await;
                }
                NetDriver::RequestResponder(rrp) => {
                    rrp.try_accept(&router, runtime_state.clone()).await;
                }
            }
        }
    }

    Ok(())
}

// TODO: Turn into service
async fn tx_network(
    mut network: Vec<NetDriver>,
    interface: String,
    runtime_state: SharedOperandState,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> std::io::Result<()> {
    use crate::driver::net::J1939Unit;

    log::debug!("Starting J1939 service on {}", interface);

    let socket = crate::net::CANSocket::bind(&crate::net::SockAddrCAN::new(&interface))?;
    let router = crate::net::Router::new(socket);

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(10));

    while shutdown.is_empty() {
        interval.tick().await;

        for driver in network.iter_mut() {
            match driver {
                NetDriver::KueblerEncoder(enc) => {
                    enc.tick(&router, runtime_state.clone()).await;
                }
                NetDriver::KueblerInclinometer(imu) => {
                    imu.tick(&router, runtime_state.clone()).await;
                }
                NetDriver::EngineManagementSystem(ems) => {
                    ems.tick(&router, runtime_state.clone()).await;
                }
                NetDriver::HydraulicControlUnit(hcu) => {
                    hcu.tick(&router, runtime_state.clone()).await;
                }
                NetDriver::RequestResponder(rrp) => {
                    rrp.tick(&router, runtime_state.clone()).await;
                }
            }
        }
    }

    Ok(())
}
