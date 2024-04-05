use std::time::Duration;

use crate::{
    driver::net::{J1939Unit, NetDriver, NetDriverCollection},
    net::ControlNetwork,
    runtime::{Service, ServiceContext, SharedOperandState},
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
    /// Vehicle system instance.
    pub vehicle_system_instance: u8,
    /// Industry group.
    pub industry_group: u8,
}

impl From<J1939Name> for j1939::Name {
    fn from(name: J1939Name) -> Self {
        j1939::NameBuilder::default()
            .identity_number(0x1)
            .manufacturer_code(name.manufacturer_code)
            .function_instance(name.function_instance)
            .ecu_instance(name.ecu_instance)
            .function(name.function)
            .vehicle_system(name.vehicle_system)
            .vehicle_system_instance(name.vehicle_system_instance)
            .industry_group(name.industry_group)
            .build()
    }
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct CanDriverConfig {
    /// Driver destination.
    pub da: u8,
    /// Driver source.
    pub sa: Option<u8>,
    /// Vendor.
    pub vendor: String,
    /// Product.
    pub product: String,
}

impl CanDriverConfig {
    pub fn to_net_driver(&self, default_da: u8) -> Result<NetDriver, ()> {
        NetDriver::factory(
            &self.vendor,
            &self.product,
            self.da,
            self.sa.unwrap_or(default_da),
        )
    }
}

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct NetworkConfig {
    /// CAN network interface.
    pub interface: String,
    /// Address.
    pub address: u8,
    /// Unit update interval.
    #[serde(default = "NetworkConfig::default_interval")]
    pub interval: u64,
    /// Network async transmit.
    #[serde(default)]
    pub authority_atx: bool,
    /// Name.
    pub name: J1939Name,
    /// Driver configuration.
    pub driver: Vec<CanDriverConfig>,
}

impl NetworkConfig {
    fn default_interval() -> u64 {
        10
    }
}

pub struct NetworkAuthorityRx {
    interface: String,
    network: ControlNetwork,
    drivers: NetDriverCollection,
}

impl Service<NetworkConfig> for NetworkAuthorityRx {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        let network = ControlNetwork::bind(&config.interface, &config.name.into()).unwrap();

        let mut drivers = NetDriverCollection::default();
        drivers.register_driver(NetDriver::VehicleManagementSystem(
            crate::driver::VehicleManagementSystem::new(config.address),
        ));

        config.driver.iter().for_each(|driver| {
            drivers.register_driver(
                driver
                    .to_net_driver(config.address)
                    .expect("Failed to register driver"),
            );
        });

        Self {
            interface: config.interface,
            network,
            drivers,
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("authority_rx", Some(self.interface.clone()))
    }

    async fn setup(&mut self, runtime_state: SharedOperandState) {
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
    }

    async fn teardown(&mut self, runtime_state: SharedOperandState) {
        for (drv, ctx) in self.drivers.inner_mut().iter_mut() {
            log::debug!(
                "[{}:0x{:X}] Teardown network driver '{}'",
                self.interface,
                drv.destination(),
                drv.name()
            );
            if let Err(error) = drv
                .teardown(ctx, &self.network, runtime_state.clone())
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

    async fn wait_io(&mut self, runtime_state: SharedOperandState) {
        if let Err(e) = self
            .network
            .listen_timeout(Duration::from_millis(100))
            .await
        {
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
}

pub struct NetworkAuthorityTx {
    interface: String,
    network: ControlNetwork,
    drivers: NetDriverCollection,
}

impl Service<NetworkConfig> for NetworkAuthorityTx {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        let network = ControlNetwork::bind(&config.interface, &config.name.into()).unwrap();

        let mut drivers = NetDriverCollection::default();
        config.driver.iter().for_each(|driver| {
            drivers.register_driver(
                driver
                    .to_net_driver(config.address)
                    .expect("Failed to register driver"),
            );
        });

        Self {
            interface: config.interface,
            network,
            drivers,
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("authority_tx", Some(self.interface.clone()))
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
    network: ControlNetwork,
    drivers: NetDriverCollection,
}

impl Service<NetworkConfig> for NetworkAuthorityAtx {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        let network = ControlNetwork::bind(&config.interface, &config.name.into()).unwrap();

        let mut drivers = NetDriverCollection::default();
        config.driver.iter().for_each(|driver| {
            drivers.register_driver(
                driver
                    .to_net_driver(config.address)
                    .expect("Failed to register driver"),
            );
        });

        Self {
            interface: config.interface,
            network,
            drivers,
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("authority_atx", Some(self.interface.clone()))
    }

    // TODO: Motion should be replaced by a more generic message type.
    async fn on_event(&mut self, runtime_state: SharedOperandState, motion: &crate::core::Motion) {
        for (drv, ctx) in self.drivers.inner_mut().iter_mut() {
            if let Err(error) = drv
                .trigger(ctx, &self.network, runtime_state.clone(), motion)
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
