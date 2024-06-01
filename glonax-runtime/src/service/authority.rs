use j1939::protocol;

use crate::{
    core::Object,
    driver::net::J1939Unit,
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

// impl CanDriverConfig {
//     pub fn to_net_driver(&self, default_da: u8) -> Result<NetDriver, ()> {
//         NetDriver::factory(
//             &self.vendor,
//             &self.product,
//             self.da,
//             self.sa.unwrap_or(default_da),
//         )
//     }
// }

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

struct NetDriverItem {
    driver: Box<dyn crate::driver::net::J1939Unit>,
    context: crate::driver::net::NetDriverContext,
}

impl NetDriverItem {
    fn new<T: J1939Unit + 'static>(driver: T) -> Self {
        Self {
            driver: Box::new(driver),
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
    default_address: u8,
    drivers: Vec<NetDriverItem>,
    is_setup: bool,
}

impl NetworkAuthority {
    async fn setup_delayed(&mut self) {
        for driver in self.drivers.iter_mut() {
            let mut tx_queue = Vec::new();

            debug!(
                "[{}] Setup network driver '{}'",
                self.network.interface(),
                driver
            );

            if let Err(error) = driver.driver.setup(&mut driver.context, &mut tx_queue) {
                error!("[{}] {}: {}", self.network.interface(), driver, error);
            }

            if let Err(e) = self.network.send_vectored(&tx_queue).await {
                error!("Failed to send vectored: {}", e);
            };
        }
    }
}

impl Clone for NetworkAuthority {
    fn clone(&self) -> Self {
        let network = ControlNetwork::bind(self.network.interface(), self.network.name()).unwrap();

        let mut drivers = Vec::new();

        for driver in &self.drivers {
            match (driver.driver.vendor(), driver.driver.product()) {
                ("laixer", "vcu") => {
                    drivers.push(NetDriverItem {
                        driver: Box::new(crate::driver::VehicleControlUnit::new(
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                    });
                }
                ("laixer", "hcu") => {
                    drivers.push(NetDriverItem {
                        driver: Box::new(crate::driver::HydraulicControlUnit::new(
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                    });
                }
                ("volvo", "d7e") => {
                    drivers.push(NetDriverItem {
                        driver: Box::new(crate::driver::VolvoD7E::new(
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                    });
                }
                ("kübler", "inclinometer") => {
                    drivers.push(NetDriverItem {
                        driver: Box::new(crate::driver::KueblerInclinometer::new(
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                    });
                }
                ("kübler", "encoder") => {
                    drivers.push(NetDriverItem {
                        driver: Box::new(crate::driver::KueblerEncoder::new(
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                    });
                }
                _ => {
                    // error!("Unknown driver: {} {}", driver.vendor, driver.product);
                    panic!()
                }
            }
        }

        Self {
            network,
            default_address: self.default_address,
            drivers,
            is_setup: self.is_setup,
        }
    }
}

impl NetworkService for NetworkAuthority {
    async fn setup2(&mut self) {
        let frame = &protocol::address_claimed(self.default_address, self.network.name());

        if let Err(e) = self.network.send(frame).await {
            error!("Failed to send address claimed: {}", e);
        }
    }

    async fn recv(&mut self, signal_tx: SignalSender) {
        if let Err(e) = self.network.recv().await {
            error!("Failed to receive from router: {}", e);
        }

        if !self.is_setup {
            self.setup_delayed().await;
            self.is_setup = true;
        }

        let frame = self.network.frame().unwrap();
        if frame.id().pgn() == j1939::PGN::Request {
            if frame.id().destination_address() != Some(self.default_address) {
                return;
            }

            let pgn = protocol::request_from_pdu(frame.pdu());
            match pgn {
                j1939::PGN::AddressClaimed => {
                    let frame =
                        protocol::address_claimed(self.default_address, self.network.name());

                    if let Err(e) = self.network.send(&frame).await {
                        error!("Failed to send address claimed: {}", e);
                    }
                }
                j1939::PGN::SoftwareIdentification => {
                    let id = j1939::IdBuilder::from_pgn(j1939::PGN::SoftwareIdentification)
                        .sa(self.default_address)
                        .build();

                    // TODO: Move this to consts
                    let version_major: u8 = crate::consts::VERSION_MAJOR.parse().unwrap();
                    let version_minor: u8 = crate::consts::VERSION_MINOR.parse().unwrap();
                    let version_patch: u8 = crate::consts::VERSION_PATCH.parse().unwrap();

                    let frame = j1939::FrameBuilder::new(id)
                        .copy_from_slice(&[1, version_major, version_minor, version_patch, b'*'])
                        .build();

                    if let Err(e) = self.network.send(&frame).await {
                        error!("Failed to send software identification: {}", e);
                    }
                }
                j1939::PGN::TimeDate => {
                    use chrono::prelude::*;

                    let utc = chrono::Utc::now();
                    let timedate = j1939::spn::TimeDate {
                        year: utc.year(),
                        month: utc.month(),
                        day: utc.day(),
                        hour: utc.hour(),
                        minute: utc.minute(),
                        second: utc.second(),
                    };

                    let id = j1939::IdBuilder::from_pgn(j1939::PGN::TimeDate)
                        .sa(self.default_address)
                        .build();

                    let frame = j1939::FrameBuilder::new(id)
                        .copy_from_slice(&timedate.to_pdu())
                        .build();

                    if let Err(e) = self.network.send(&frame).await {
                        error!("Failed to send time date: {}", e);
                    }
                }
                _ => (),
            }

            return;
        }

        // TODO: Only send frames to drivers that are interested in them.
        for driver in self.drivers.iter_mut() {
            if let Err(error) =
                driver
                    .driver
                    .try_accept(&mut driver.context, &self.network, signal_tx.clone())
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

    async fn teardown2(&mut self) {
        for driver in self.drivers.iter_mut() {
            let mut tx_queue = Vec::new();

            debug!(
                "[{}] Teardown network driver '{}'",
                self.network.interface(),
                driver
            );

            if let Err(error) = driver.driver.teardown(&mut driver.context, &mut tx_queue) {
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

        // TODO: Move this driver thing to a factory.
        let mut drivers = Vec::new();
        for driver in config.driver.iter() {
            match (driver.vendor.as_str(), driver.product.as_str()) {
                ("laixer", "vcu") => {
                    drivers.push(NetDriverItem::new(crate::driver::VehicleControlUnit::new(
                        driver.da,
                        driver.sa.unwrap_or(config.address),
                    )));
                }
                ("laixer", "hcu") => {
                    drivers.push(NetDriverItem::new(
                        crate::driver::HydraulicControlUnit::new(
                            driver.da,
                            driver.sa.unwrap_or(config.address),
                        ),
                    ));
                }
                ("volvo", "d7e") => {
                    drivers.push(NetDriverItem::new(crate::driver::VolvoD7E::new(
                        driver.da,
                        driver.sa.unwrap_or(config.address),
                    )));
                }
                ("kübler", "inclinometer") => {
                    drivers.push(NetDriverItem::new(crate::driver::KueblerInclinometer::new(
                        driver.da,
                        driver.sa.unwrap_or(config.address),
                    )));
                }
                // TODO:
                // ("j1939", "ecm") => {
                //     drivers.push(NetDriverItem::new(
                //         crate::driver::EngineManagementSystem::new(
                //             driver.da,
                //             driver.sa.unwrap_or(config.address),
                //         ),
                //     ));
                // }
                ("kübler", "encoder") => {
                    drivers.push(NetDriverItem::new(crate::driver::KueblerEncoder::new(
                        driver.da,
                        driver.sa.unwrap_or(config.address),
                    )));
                }
                _ => {
                    error!("Unknown driver: {} {}", driver.vendor, driver.product);
                }
            }
        }

        Self {
            network,
            default_address: config.address,
            drivers,
            is_setup: false,
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::with_address("authority", self.network.interface())
    }

    async fn setup(&mut self) {
        self.setup2().await;
    }

    async fn teardown(&mut self) {
        self.teardown2().await;
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
