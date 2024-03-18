mod error;

use crate::{
    core::{Instance, Target},
    driver::net::{NetDriver, NetDriverContext},
    world::World,
    MachineState,
};

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

pub type MotionSender = tokio::sync::mpsc::Sender<crate::core::Motion>;
pub type MotionReceiver = tokio::sync::mpsc::Receiver<crate::core::Motion>;
pub type SharedOperandState = std::sync::Arc<tokio::sync::RwLock<crate::Operand>>;

pub mod builder;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NullConfig;

pub struct ControlNetwork {
    pub default_source_address: u8,
    pub network: Vec<(NetDriver, NetDriverContext)>,
}

impl ControlNetwork {
    pub fn new(default_source_address: u8) -> Self {
        Self {
            default_source_address,
            network: vec![],
        }
    }

    pub fn with_request_responder(address: u8) -> Self {
        Self {
            default_source_address: address,
            network: vec![(
                NetDriver::request_responder(address),
                NetDriverContext {
                    tx_last: std::time::Instant::now(),
                    rx_last: std::time::Instant::now(),
                },
            )],
        }
    }

    pub fn register_driver(&mut self, driver: NetDriver) {
        self.network.push((
            driver,
            NetDriverContext {
                tx_last: std::time::Instant::now(),
                rx_last: std::time::Instant::now(),
            },
        ));
    }
}

pub struct ServiceContext {
    /// Service name.
    name: String,
    /// Service address.
    address: Option<String>,
}

impl ServiceContext {
    pub fn new(name: impl ToString, address: Option<impl ToString>) -> Self {
        Self {
            name: name.to_string(),
            address: address.map(|a| a.to_string()),
        }
    }
}

// TODO: Change to ServiceContext
pub struct ServiceErrorBuilder {
    name: String,
    address: String,
}

impl ServiceErrorBuilder {
    pub fn new(name: impl ToString, address: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            address: address.to_string(),
        }
    }

    pub fn io_error(&self, io_error: std::io::Error) -> ServiceError {
        ServiceError {
            name: self.name.clone(),
            address: self.address.clone(),
            io_error,
        }
    }
}

pub struct ServiceError {
    name: String,
    address: String,
    io_error: std::io::Error,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Service {} error: {}", self.name, self.io_error)
    }
}

impl std::fmt::Debug for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Service {} error: {:?}", self.name, self.io_error)
    }
}

impl std::error::Error for ServiceError {}

pub trait Service<Cnf> {
    // TODO: Add instance to new
    /// Construct a new component.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    fn ctx(&self) -> ServiceContext {
        ServiceContext {
            name: std::any::type_name::<Self>().to_string(),
            address: None,
        }
    }

    /// Wait for IO event.
    fn wait_io(
        &mut self,
        _runtime_state: SharedOperandState,
    ) -> impl std::future::Future<Output = ()> + Send {
        std::future::ready(())
    }

    fn tick(&mut self, _runtime_state: SharedOperandState) {}
}

pub trait Component<Cnf: Clone> {
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
pub fn builder<Cnf: Clone>(
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

impl<Cnf: Clone + Send + 'static> Runtime<Cnf> {
    /// Listen for shutdown signal.
    #[inline]
    pub fn shutdown_signal(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown.0.subscribe()
    }

    /// Listen for network service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the operand and the interface name.
    pub fn schedule_j1939_service_rx(&self, network: ControlNetwork, interface: &str) {
        let operand = self.operand.clone();
        let interface = interface.to_owned();
        let shutdown = self.shutdown.0.subscribe();

        tokio::spawn(async move {
            if let Err(e) = rx_network(network.network, interface, operand, shutdown).await {
                log::error!("Failed to start network service: {}", e);
            }
        });
    }

    pub fn schedule_j1939_service_tx(&self, network: ControlNetwork, interface: &str) {
        let operand = self.operand.clone();
        let interface = interface.to_owned();
        let shutdown = self.shutdown.0.subscribe();

        tokio::spawn(async move {
            if let Err(e) = tx_network(network.network, interface, operand, shutdown).await {
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

    //
    // Services
    //

    // TODO: Remove this method
    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_func<Fut>(
        &self,
        service: impl FnOnce(Cnf, Instance, SharedOperandState, MotionSender) -> Fut + Send + 'static,
    ) where
        Fut: std::future::Future<Output = std::result::Result<(), ServiceError>> + Send + 'static,
    {
        let config = self.config.clone();
        let instance = self.instance.clone();
        let operand = self.operand.clone();
        let motion_tx = self.motion_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = service(config, instance, operand, motion_tx).await {
                log::error!(
                    "Failed to schedule '{}' at {}: {}",
                    e.name,
                    e.address,
                    e.io_error
                );
            }
        });
    }

    // TODO: Services should be able to return a result
    /// Listen for IO event service in the background.
    ///
    /// This method will spawn a service in the background and return immediately. The service
    /// will be provided with a copy of the runtime configuration and a reference to the runtime.
    pub fn schedule_io_service<S, C>(&self, config: C)
    where
        S: Service<C> + Send + Sync + 'static,
        C: std::clone::Clone,
    {
        let mut service = S::new(config.clone());
        let ctx = service.ctx();

        let operand = self.operand.clone();

        if let Some(address) = ctx.address.clone() {
            log::debug!("Starting {} service on {}", ctx.name, address,);
        } else {
            log::debug!("Starting {} service", ctx.name);
        }

        tokio::spawn(async move {
            service.wait_io(operand).await;
        });
    }

    /// Schedule a component to run in the background.
    ///
    /// This method will schedule a component to run in the background. On each tick, the component
    /// will be provided with a component context and a mutable reference to the runtime state.
    pub fn schedule_service<S, C>(&self, config: C, duration: std::time::Duration)
    where
        S: Service<C> + Send + Sync + 'static,
        C: std::clone::Clone,
    {
        let mut interval = tokio::time::interval(duration);

        let mut service = S::new(config.clone());
        let ctx = service.ctx();

        let operand = self.operand.clone();

        if let Some(address) = ctx.address.clone() {
            log::debug!("Starting {} service on {}", ctx.name, address,);
        } else {
            log::debug!("Starting {} service", ctx.name);
        }

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                service.tick(operand.clone());
            }
        });
    }

    // TODO: Component should be 'service' and not 'component'
    // TODO: Maybe copy MachineState to component state on each tick?
    /// Run a component in the main thread.
    ///
    /// This method will run a component in the main thread until the runtime is shutdown.
    /// On each tick, the component will be provided with a component context and a mutable
    /// reference to the runtime state.
    pub async fn run_interval<C>(&self, mut component: C, duration: std::time::Duration)
    where
        C: Component<Cnf>,
        Cnf: Clone,
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
    mut network: Vec<(NetDriver, NetDriverContext)>,
    interface: String,
    runtime_state: SharedOperandState,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> std::io::Result<()> {
    use crate::driver::net::{J1939Unit, J1939UnitOperationState};

    log::debug!("Starting J1939 service on {}", interface);

    let socket = crate::net::CANSocket::bind(&crate::net::SockAddrCAN::new(&interface))?;
    let mut router = crate::net::Router::new(socket);

    router
        .send(&j1939::protocol::address_claimed(
            router.source_address(),
            *router.name(),
        ))
        .await?;

    let state = J1939UnitOperationState::Setup;
    for (drv, ctx) in network.iter_mut() {
        let result = match drv {
            NetDriver::KueblerEncoder(enc) => {
                enc.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::KueblerInclinometer(imu) => {
                imu.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::VolvoD7E(ems) => {
                ems.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::BoschEngineManagementSystem(ems) => {
                ems.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::HydraulicControlUnit(hcu) => {
                hcu.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::RequestResponder(rrp) => {
                rrp.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
        };

        if let Err(e) = result {
            log::error!("Failed to setup network: {}", e);
        }
    }

    while shutdown.is_empty() {
        if let Err(e) = router.listen().await {
            log::error!("Failed to receive from router: {}", e);
        }

        let state = J1939UnitOperationState::Running;
        for (drv, ctx) in network.iter_mut() {
            match drv {
                NetDriver::KueblerEncoder(enc) => {
                    enc.try_accept(ctx, &state, &router, runtime_state.clone())
                        .await;
                }
                NetDriver::KueblerInclinometer(imu) => {
                    imu.try_accept(ctx, &state, &router, runtime_state.clone())
                        .await;
                }
                NetDriver::VolvoD7E(ems) => {
                    ems.try_accept(ctx, &state, &router, runtime_state.clone())
                        .await;
                }
                NetDriver::BoschEngineManagementSystem(ems) => {
                    ems.try_accept(ctx, &state, &router, runtime_state.clone())
                        .await;
                }
                NetDriver::HydraulicControlUnit(hcu) => {
                    let res = hcu
                        .try_accept(ctx, &state, &router, runtime_state.clone())
                        .await;

                    if let Err(e) = res {
                        log::error!("[{}:0x4A] Hydraulic control: {}", interface, e);
                    }
                }
                NetDriver::RequestResponder(rrp) => {
                    rrp.try_accept(ctx, &state, &router, runtime_state.clone())
                        .await;
                }
            }

            // if let Err(e) = result {
            //     log::error!("[{}:0x4A] Hydraulic control unit ingress setup: {}", e);
            // }
        }
    }

    let state = J1939UnitOperationState::Teardown;
    for (drv, ctx) in network.iter_mut() {
        let result = match drv {
            NetDriver::KueblerEncoder(enc) => {
                enc.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::KueblerInclinometer(imu) => {
                imu.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::VolvoD7E(ems) => {
                ems.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::BoschEngineManagementSystem(ems) => {
                ems.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::HydraulicControlUnit(hcu) => {
                hcu.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::RequestResponder(rrp) => {
                rrp.try_accept(ctx, &state, &router, runtime_state.clone())
                    .await
            }
        };

        if let Err(e) = result {
            log::error!("Failed to tick network: {}", e);
        }
    }

    Ok(())
}

// TODO: Turn into service
async fn tx_network(
    mut network: Vec<(NetDriver, NetDriverContext)>,
    interface: String,
    runtime_state: SharedOperandState,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> std::io::Result<()> {
    use crate::driver::net::{J1939Unit, J1939UnitOperationState};

    log::debug!("Starting J1939 service on {}", interface);

    let socket = crate::net::CANSocket::bind(&crate::net::SockAddrCAN::new(&interface))?;
    let router = crate::net::Router::new(socket);

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(10));

    let state = J1939UnitOperationState::Setup;
    for (drv, ctx) in network.iter_mut() {
        let result = match drv {
            NetDriver::KueblerEncoder(enc) => {
                enc.tick(ctx, &state, &router, runtime_state.clone()).await
            }
            NetDriver::KueblerInclinometer(imu) => {
                imu.tick(ctx, &state, &router, runtime_state.clone()).await
            }
            NetDriver::VolvoD7E(ems) => ems.tick(ctx, &state, &router, runtime_state.clone()).await,
            NetDriver::BoschEngineManagementSystem(ems) => {
                ems.tick(ctx, &state, &router, runtime_state.clone()).await
            }
            NetDriver::HydraulicControlUnit(hcu) => {
                hcu.tick(ctx, &state, &router, runtime_state.clone()).await
            }
            NetDriver::RequestResponder(rrp) => {
                rrp.tick(ctx, &state, &router, runtime_state.clone()).await
            }
        };

        if let Err(e) = result {
            log::error!("Failed to tick network: {}", e);
        }
    }

    while shutdown.is_empty() {
        interval.tick().await;

        let state = J1939UnitOperationState::Running;
        for (drv, ctx) in network.iter_mut() {
            match drv {
                NetDriver::KueblerEncoder(enc) => {
                    let res = enc.tick(ctx, &state, &router, runtime_state.clone()).await;

                    if let Err(e) = res {
                        log::error!("[{}:0x0] Kuebler encoder: {}", interface, e);
                    }
                }
                NetDriver::KueblerInclinometer(imu) => {
                    let res = imu.tick(ctx, &state, &router, runtime_state.clone()).await;

                    if let Err(e) = res {
                        log::error!("[{}:0x0] Kuebler inclinometer: {}", interface, e);
                    }
                }
                NetDriver::VolvoD7E(ems) => {
                    let res = ems.tick(ctx, &state, &router, runtime_state.clone()).await;

                    if let Err(e) = res {
                        log::error!("[{}:0x0] Volvo D7E: {}", interface, e);
                    }
                }
                NetDriver::BoschEngineManagementSystem(ems) => {
                    let res = ems.tick(ctx, &state, &router, runtime_state.clone()).await;

                    if let Err(e) = res {
                        log::error!("[{}:0x0] Bosch engine management system: {}", interface, e);
                    }
                }
                NetDriver::HydraulicControlUnit(hcu) => {
                    let res = hcu.tick(ctx, &state, &router, runtime_state.clone()).await;

                    if let Err(e) = res {
                        log::error!("[{}:0x4A] Hydraulic control: {}", interface, e);
                    }
                }
                NetDriver::RequestResponder(rrp) => {
                    let res = rrp.tick(ctx, &state, &router, runtime_state.clone()).await;

                    if let Err(e) = res {
                        log::error!("[{}:0x0] Request responder: {}", interface, e);
                    }
                }
            }

            // if let Err(e) = result {
            //     log::error!("Failed to tick network: {}", e);
            // }
        }
    }

    let state = J1939UnitOperationState::Teardown;
    for (drv, ctx) in network.iter_mut() {
        let result = match drv {
            NetDriver::KueblerEncoder(enc) => {
                enc.tick(ctx, &state, &router, runtime_state.clone()).await
            }
            NetDriver::KueblerInclinometer(imu) => {
                imu.tick(ctx, &state, &router, runtime_state.clone()).await
            }
            NetDriver::VolvoD7E(ems) => ems.tick(ctx, &state, &router, runtime_state.clone()).await,
            NetDriver::BoschEngineManagementSystem(ems) => {
                ems.tick(ctx, &state, &router, runtime_state.clone()).await
            }
            NetDriver::HydraulicControlUnit(hcu) => {
                hcu.tick(ctx, &state, &router, runtime_state.clone()).await
            }
            NetDriver::RequestResponder(rrp) => {
                rrp.tick(ctx, &state, &router, runtime_state.clone()).await
            }
        };

        if let Err(e) = result {
            log::error!("Failed to tick network: {}", e);
        }
    }

    Ok(())
}

// TODO: Turn into service
pub async fn atx_network_1(
    interface: String,
    runtime_state: SharedOperandState,
    mut motion_rx: MotionReceiver,
) -> std::io::Result<()> {
    use crate::driver::net::{J1939Unit, J1939UnitOperationState};

    log::debug!("Starting J1939 ATX service on {}", interface);

    /// Vehicle Management System J1939 address.
    const J1939_ADDRESS_VMS: u8 = 0x27;
    /// Hydraulic Control Unit 0 J1939 address.
    const J1939_ADDRESS_HCU0: u8 = 0x4a;

    let socket = crate::net::CANSocket::bind(&crate::net::SockAddrCAN::new(&interface))?;
    let router = crate::net::Router::new(socket);

    let hcu0 = crate::driver::HydraulicControlUnit::new(J1939_ADDRESS_HCU0, J1939_ADDRESS_VMS);

    let mut network = ControlNetwork::new(J1939_ADDRESS_VMS);
    network.register_driver(NetDriver::HydraulicControlUnit(hcu0));

    let state = J1939UnitOperationState::Setup;
    for (drv, ctx) in network.network.iter_mut() {
        let result = match drv {
            NetDriver::KueblerEncoder(enc) => {
                enc.trigger(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::KueblerInclinometer(imu) => {
                imu.trigger(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::VolvoD7E(ems) => {
                ems.trigger(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::BoschEngineManagementSystem(ems) => {
                ems.trigger(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::HydraulicControlUnit(hcu) => {
                hcu.trigger(ctx, &state, &router, runtime_state.clone())
                    .await
            }
            NetDriver::RequestResponder(rrp) => {
                rrp.trigger(ctx, &state, &router, runtime_state.clone())
                    .await
            }
        };

        if let Err(e) = result {
            log::error!("Failed to tick network: {}", e);
        }
    }

    while let Some(motion) = motion_rx.recv().await {
        // TODO: Move
        runtime_state.write().await.state.motion = motion.clone();

        let state = J1939UnitOperationState::Running;
        for (drv, ctx) in network.network.iter_mut() {
            let result = match drv {
                NetDriver::KueblerEncoder(enc) => {
                    enc.trigger(ctx, &state, &router, runtime_state.clone())
                        .await
                }
                NetDriver::KueblerInclinometer(imu) => {
                    imu.trigger(ctx, &state, &router, runtime_state.clone())
                        .await
                }
                NetDriver::VolvoD7E(ems) => {
                    ems.trigger(ctx, &state, &router, runtime_state.clone())
                        .await
                }
                NetDriver::BoschEngineManagementSystem(ems) => {
                    ems.trigger(ctx, &state, &router, runtime_state.clone())
                        .await
                }
                NetDriver::HydraulicControlUnit(hcu) => {
                    hcu.trigger(ctx, &state, &router, runtime_state.clone())
                        .await
                }
                NetDriver::RequestResponder(rrp) => {
                    rrp.trigger(ctx, &state, &router, runtime_state.clone())
                        .await
                }
            };

            if let Err(e) = result {
                log::error!("Failed to tick network: {}", e);
            }
        }
    }

    Ok(())
}
