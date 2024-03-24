use crate::{driver::net::J1939Unit, runtime::{Service, SharedOperandState}};

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct J1939Name {
    /// Manufacturer code.
    pub manufacturer_code: u16,
    /// Function instance.
    pub function_instance: u8,
    /// ECU instance.
    pub ecu_instance: u8,
    /// Function.
    pub function: u8,
    /// Vehicle system.
    pub vehicle_system: u8,
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct CanDriverConfig {
    /// Driver destination.
    pub da: u8,
    /// Driver source.
    pub sa: Option<u8>,
    /// Driver type.
    #[serde(rename = "type")]
    pub driver_type: String,
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct NetworkConfig {
    /// CAN network interface.
    pub interface: String,
    /// Address.
    pub address: u8,
    /// Name.
    pub name: J1939Name,
    /// Driver configuration.
    pub driver: Vec<CanDriverConfig>,
}

pub struct NetworkAuthorityRx {
    interface: String,
    router: crate::net::Router,
    network: crate::runtime::ControlNetwork,
}

impl Service<NetworkConfig> for NetworkAuthorityRx {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        // TODO: Let the router bind to a socket.
        let socket = crate::net::CANSocket::bind(&crate::net::SockAddrCAN::new(&config.interface)).unwrap();
        let router = crate::net::Router::new(socket);

        let mut network = crate::runtime::ControlNetwork::with_request_responder(config.address);
        for driver in &config.driver {
            // TODO: Support the 'into' trait.
            let net_driver_config = crate::driver::net::NetDriverConfig {
                driver_type: driver.driver_type.clone(),
                destination: driver.da,
                source: driver.sa.unwrap_or(config.address), // TODO: Maybe remove 'source' from config.
            };

            network.register_driver(crate::driver::net::NetDriver::try_from(net_driver_config).unwrap());
        }

        Self { interface: config.interface, router, network }
    }

    fn ctx(&self) -> crate::runtime::ServiceContext {
        crate::runtime::ServiceContext::new("authority_rx", Some(self.interface.clone()))
    }

    async fn wait_io(&mut self, runtime_state: SharedOperandState) {
        // FUTURE: Could move this into a virtual ECU that represents the VMS.
        self.router.send(&j1939::protocol::address_claimed(self.router.source_address(), *self.router.name())).await.unwrap();

        for (drv, ctx) in self.network.network.iter_mut() {
            log::debug!("[{}:0x{:X}] Setup network driver '{}'", self.interface, drv.destination(), drv.name());
            if let Err(error) = drv.setup(ctx, &self.router, runtime_state.clone()).await {
                log::error!("[{}:0x{:X}] {}: {}", self.interface, drv.destination(), drv.name(), error);
            }
        }

        // TODO: Replace with: while shutdown.is_empty()
        loop {
            // if let Ok(Err(e)) =
            //     tokio::time::timeout(std::time::Duration::from_millis(100), router.listen()).await
            // {    
            //     log::error!("Failed to receive from router: {}", e);
            // }

            if let Err(e) = self.router.listen().await {
                log::error!("Failed to receive from router: {}", e);
            }

            for (drv, ctx) in self.network.network.iter_mut() {
                if let Err(error) = drv.try_accept(ctx, &self.router, runtime_state.clone()).await {
                    log::error!("[{}:0x{:X}] {}: {}", self.interface, drv.destination(), drv.name(), error);
                }
            }
        }

        // for (drv, ctx) in self.network.network.iter_mut() {
        //     log::debug!("[{}:0x{:X}] Teardown network driver '{}'", self.interface, drv.destination(), drv.name());
        //     if let Err(error) = drv.teardown(ctx, &self.router, runtime_state.clone()).await {
        //         log::error!("[{}:0x{:X}] {}: {}", self.interface, drv.destination(), drv.name(), error);
        //     }
        // }
    }
}

pub struct NetworkAuthorityTx {
    interface: String,
    router: crate::net::Router,
    network: crate::runtime::ControlNetwork,
}

impl Service<NetworkConfig> for NetworkAuthorityTx {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        let socket = crate::net::CANSocket::bind(&crate::net::SockAddrCAN::new(&config.interface)).unwrap();
        let router = crate::net::Router::new(socket);

        let mut network = crate::runtime::ControlNetwork::new(config.address);
        for driver in &config.driver {
            let net_driver_config = crate::driver::net::NetDriverConfig {
                driver_type: driver.driver_type.clone(),
                destination: driver.da,
                source: driver.sa.unwrap_or(config.address), // TODO: Maybe remove 'source' from config.
            };

            network.register_driver(crate::driver::net::NetDriver::try_from(net_driver_config).unwrap());
        }

        Self { interface: config.interface, router, network }
    }

    fn ctx(&self) -> crate::runtime::ServiceContext {
        crate::runtime::ServiceContext::new("authority_tx", Some(self.interface.clone()))
    }

    async fn tick(&mut self, runtime_state: SharedOperandState) {
        for (drv, ctx) in self.network.network.iter_mut() {
            if let Err(error) = drv.tick(ctx, &self.router, runtime_state.clone()).await {
                log::error!("[{}:0x{:X}] {}: {}", self.interface, drv.destination(), drv.name(), error);
            }
        }
    }
}

pub struct NetworkAuthorityAtx {
    interface: String,
    router: crate::net::Router,
    network: crate::runtime::ControlNetwork,
}

impl Service<NetworkConfig> for NetworkAuthorityAtx {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        let socket = crate::net::CANSocket::bind(&crate::net::SockAddrCAN::new(&config.interface)).unwrap();
        let router = crate::net::Router::new(socket);

        let mut network = crate::runtime::ControlNetwork::new(config.address);
        // for driver in &config.driver {
        //     let net_driver_config = crate::driver::net::NetDriverConfig {
        //         driver_type: driver.driver_type.clone(),
        //         destination: driver.da,
        //         source: driver.sa.unwrap_or(config.address), // TODO: Maybe remove 'source' from config.
        //     };

        //     network.register_driver(crate::driver::net::NetDriver::try_from(net_driver_config).unwrap());
        // }

        // TODO: Get from config.
        let hcu0 = crate::driver::HydraulicControlUnit::new(0x4a, config.address);
        network.register_driver(crate::driver::net::NetDriver::HydraulicControlUnit(hcu0));

        Self { interface: config.interface, router, network }
    }

    fn ctx(&self) -> crate::runtime::ServiceContext {
        crate::runtime::ServiceContext::new("authority_atx", Some(self.interface.clone()))
    }

    // TODO: This hasnt been been worked out but the queue must be awaited in the runtime.
    // TODO: Motion should be replaced by a more generic message type.
    async fn on_event(&mut self, runtime_state: SharedOperandState, mut motion_rx: crate::runtime::MotionReceiver) {
        while let Some(motion) = motion_rx.recv().await {
            runtime_state.write().await.state.motion = motion.clone();

            for (drv, ctx) in self.network.network.iter_mut() {
                if let Err(error) = drv.trigger(ctx, &self.router, runtime_state.clone(), &motion).await {
                    log::error!("[{}:0x{:X}] {}: {}", self.interface, drv.destination(), drv.name(), error);
                }
            }
        }
    }
}
