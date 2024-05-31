use crate::{
    core::Object,
    driver::net::{J1939Unit, NetDriver},
    net::ControlNetwork,
    runtime::{NetworkService, Service, ServiceContext, SignalSender},
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

#[derive(Clone)]
struct NetDriverItem {
    driver: NetDriver,
    context: crate::driver::net::NetDriverContext,
}

impl NetDriverItem {
    fn new(driver: NetDriver) -> Self {
        Self {
            driver,
            context: crate::driver::net::NetDriverContext::default(),
        }
    }
}

impl std::fmt::Display for NetDriverItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:0x{:X}",
            self.driver.name(), // TOOD: Move this further up the chain.
            self.driver.destination()
        )
    }
}

pub struct NetworkAuthority {
    network: ControlNetwork,
    drivers: Vec<NetDriverItem>,
    is_setup: bool,
}

impl NetworkAuthority {
    async fn setup_delayed(&mut self) {
        for driver in self.drivers.iter_mut() {
            debug!(
                "[{}] Setup network driver '{}'",
                self.network.interface(),
                driver
            );

            if let Err(error) = driver
                .driver
                .setup(&mut driver.context, &self.network)
                .await
            {
                error!("[{}] {}: {}", self.network.interface(), driver, error);
            }
        }
    }
}

impl Clone for NetworkAuthority {
    fn clone(&self) -> Self {
        let network = ControlNetwork::bind(self.network.interface(), self.network.name()).unwrap();

        Self {
            network,
            drivers: self.drivers.clone(),
            is_setup: self.is_setup,
        }
    }
}

impl NetworkService for NetworkAuthority {
    async fn recv(&mut self, signal_tx: SignalSender) {
        if let Err(e) = self.network.recv().await {
            error!("Failed to receive from router: {}", e);
        }

        if !self.is_setup {
            self.setup_delayed().await;
            self.is_setup = true;
        }

        for driver in self.drivers.iter_mut() {
            if let Err(error) = driver
                .driver
                .try_accept(&mut driver.context, &self.network, signal_tx.clone())
                .await
            {
                error!("[{}] {}: {}", self.network.interface(), driver, error);
            }
        }
    }

    async fn on_tick(&mut self) {
        for driver in self.drivers.iter_mut() {
            let mut tx_queue = Vec::new();

            if let Err(error) = driver.driver.tick(&mut driver.context, &mut tx_queue) {
                error!("[{}] {}: {}", self.network.interface(), driver, error);
            }

            if let Err(e) = self.network.send_vectored(&tx_queue).await {
                error!("Failed to send vectored: {}", e);
            };
        }
    }

    async fn on_command(&mut self, object: &Object) {
        for driver in self.drivers.iter_mut() {
            let mut tx_queue = Vec::new();

            if let Err(error) = driver
                .driver
                .trigger(&mut driver.context, &mut tx_queue, object)
            {
                error!("[{}] {}: {}", self.network.interface(), driver, error);
            }

            if let Err(e) = self.network.send_vectored(&tx_queue).await {
                error!("Failed to send vectored: {}", e);
            };
        }
    }
}

// TODO: Replace with `NetworkService`
impl Service<NetworkConfig> for NetworkAuthority {
    fn new(config: NetworkConfig) -> Self
    where
        Self: Sized,
    {
        let mut filter = crate::net::Filter::reject();

        filter.push(crate::net::FilterItem::SourceAddress(config.address));

        let network = ControlNetwork::bind(&config.interface, &config.name.into())
            .unwrap()
            .with_filter(filter);

        let mut drivers = Vec::new();

        drivers.push(NetDriverItem::new(NetDriver::VehicleManagementSystem(
            crate::driver::VehicleManagementSystem::new(config.address),
        )));

        for driver in config.driver.iter() {
            let net_driver = driver
                .to_net_driver(config.address)
                .expect("Failed to register driver");

            drivers.push(NetDriverItem::new(net_driver));
        }

        Self {
            network,
            drivers,
            is_setup: false,
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::with_address("authority_rx", self.network.interface())
    }

    async fn teardown(&mut self) {
        for driver in self.drivers.iter_mut() {
            debug!(
                "[{}] Teardown network driver '{}'",
                self.network.interface(),
                driver
            );

            if let Err(error) = driver
                .driver
                .teardown(&mut driver.context, &self.network)
                .await
            {
                error!("[{}] {}: {}", self.network.interface(), driver, error);
            }
        }
    }

    async fn wait_io_pub(&mut self, signal_tx: SignalSender) {
        self.recv(signal_tx).await;
    }

    async fn tick(&mut self) {
        self.on_tick().await;
    }

    async fn command(&mut self, object: &Object) {
        self.on_command(object).await;
    }
}
