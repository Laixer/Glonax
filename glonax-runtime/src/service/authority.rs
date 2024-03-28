use crate::{
    driver::net::{J1939Unit, NetDriver, NetDriverCollection},
    runtime::{Service, SharedOperandState},
};

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
    network: crate::net::ControlNetwork,
    drivers: NetDriverCollection,
}

impl Service<NetworkConfig> for NetworkAuthorityRx {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        let network = crate::net::ControlNetwork::bind(&config.interface).unwrap();

        let mut drivers = NetDriverCollection::default();
        drivers.register_driver(NetDriver::VehicleManagementSystem(
            crate::driver::VehicleManagementSystem::new(config.address),
        ));

        for driver in &config.driver {
            match NetDriver::factory(&driver.driver_type, driver.da, driver.sa.unwrap_or(config.address)) {
                Ok(driver) => {
                    drivers.register_driver(driver);
                }
                Err(()) => {
                    log::error!("Failed to register driver: {}", driver.driver_type);
                }
            }
        }

        Self {
            interface: config.interface,
            network,
            drivers,
        }
    }

    fn ctx(&self) -> crate::runtime::ServiceContext {
        crate::runtime::ServiceContext::new("authority_rx", Some(self.interface.clone()))
    }

    async fn wait_io(&mut self, runtime_state: SharedOperandState) {
        for (drv, ctx) in self.drivers.inner_mut().iter_mut() {
            log::debug!(
                "[{}:0x{:X}] Setup network driver '{}'",
                self.interface,
                drv.destination(),
                drv.name()
            );
            if let Err(error) = drv.setup(ctx, &self.network, runtime_state.clone()).await {
                log::error!(
                    "[{}:0x{:X}] {}: {}",
                    self.interface,
                    drv.destination(),
                    drv.name(),
                    error
                );
            }
        }

        // TODO: Replace with: while shutdown.is_empty()
        loop {
            // if let Ok(Err(e)) =
            //     tokio::time::timeout(std::time::Duration::from_millis(100), router.listen()).await
            // {
            //     log::error!("Failed to receive from router: {}", e);
            // }

            if let Err(e) = self.network.listen().await {
                log::error!("Failed to receive from router: {}", e);
            }

            for (drv, ctx) in self.drivers.inner_mut().iter_mut() {
                if let Err(error) = drv
                    .try_accept(ctx, &self.network, runtime_state.clone())
                    .await
                {
                    log::error!(
                        "[{}:0x{:X}] {}: {}",
                        self.interface,
                        drv.destination(),
                        drv.name(),
                        error
                    );
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
    network: crate::net::ControlNetwork,
    drivers: NetDriverCollection,
}

impl Service<NetworkConfig> for NetworkAuthorityTx {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        let network = crate::net::ControlNetwork::bind(&config.interface).unwrap();

        let mut drivers = NetDriverCollection::default();
        for driver in &config.driver {
            match NetDriver::factory(
                &driver.driver_type,
                driver.da,
                driver.sa.unwrap_or(config.address),
            ) {
                Ok(driver) => {
                    drivers.register_driver(driver);
                }
                Err(()) => {
                    log::error!("Failed to register driver: {}", driver.driver_type);
                }
            }
        }

        Self {
            interface: config.interface,
            network,
            drivers,
        }
    }

    fn ctx(&self) -> crate::runtime::ServiceContext {
        crate::runtime::ServiceContext::new("authority_tx", Some(self.interface.clone()))
    }

    async fn tick(&mut self, runtime_state: SharedOperandState) {
        for (drv, ctx) in self.drivers.inner_mut().iter_mut() {
            if let Err(error) = drv.tick(ctx, &self.network, runtime_state.clone()).await {
                log::error!(
                    "[{}:0x{:X}] {}: {}",
                    self.interface,
                    drv.destination(),
                    drv.name(),
                    error
                );
            }
        }
    }
}

pub struct NetworkAuthorityAtx {
    interface: String,
    network: crate::net::ControlNetwork,
    drivers: NetDriverCollection,
}

impl Service<NetworkConfig> for NetworkAuthorityAtx {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        let network = crate::net::ControlNetwork::bind(&config.interface).unwrap();

        let mut drivers = NetDriverCollection::default();

        // TODO: Get from config.
        let hcu0 = crate::driver::HydraulicControlUnit::new(0x4a, config.address);
        drivers.register_driver(NetDriver::HydraulicControlUnit(hcu0));

        Self {
            interface: config.interface,
            network,
            drivers,
        }
    }

    fn ctx(&self) -> crate::runtime::ServiceContext {
        crate::runtime::ServiceContext::new("authority_atx", Some(self.interface.clone()))
    }

    // TODO: This hasnt been been worked out but the queue must be awaited in the runtime.
    // TODO: Motion should be replaced by a more generic message type.
    async fn on_event(
        &mut self,
        runtime_state: SharedOperandState,
        mut motion_rx: crate::runtime::MotionReceiver,
    ) {
        while let Some(motion) = motion_rx.recv().await {
            for (drv, ctx) in self.drivers.inner_mut().iter_mut() {
                if let Err(error) = drv
                    .trigger(ctx, &self.network, runtime_state.clone(), &motion)
                    .await
                {
                    log::error!(
                        "[{}:0x{:X}] {}: {}",
                        self.interface,
                        drv.destination(),
                        drv.name(),
                        error
                    );
                }
            }
        }
    }
}
