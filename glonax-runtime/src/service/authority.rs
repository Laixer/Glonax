use std::time::Duration;

use j1939::protocol;

use crate::{
    core::Object,
    net::ControlNetwork,
    runtime::{
        J1939Unit, J1939UnitError, J1939UnitOk, NetDriverContext, NetworkService, SignalSender,
    },
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
    /// Timeout in milliseconds.
    pub timeout: Option<u64>,
    /// Vendor.
    pub vendor: String,
    /// Product.
    pub product: String,
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

struct NetDriverItem {
    driver: Box<dyn J1939Unit>,
    context: NetDriverContext,
    /// Timeout for receiving messages.
    rx_timeout: Option<Duration>,
}

impl NetDriverItem {
    fn new<T: J1939Unit + 'static>(driver: T) -> Self {
        Self {
            rx_timeout: None,
            driver: Box::new(driver),
            context: NetDriverContext::default(),
        }
    }

    pub fn is_rx_timeout(&self) -> bool {
        self.rx_timeout
            .map(|timeout| self.context.is_rx_timeout(timeout))
            .unwrap_or(false)
    }

    fn setup(&mut self, tx_queue: &mut Vec<j1939::Frame>) -> Result<(), J1939UnitError> {
        self.driver.setup(&mut self.context, tx_queue)
    }

    fn try_recv(
        &mut self,
        frame: &j1939::Frame,
        signal_tx: SignalSender,
    ) -> Result<J1939UnitOk, J1939UnitError> {
        self.driver.try_recv(&mut self.context, frame, signal_tx)
    }

    fn tick(&mut self, tx_queue: &mut Vec<j1939::Frame>) -> Result<(), J1939UnitError> {
        self.driver.tick(&mut self.context, tx_queue)
    }

    fn trigger(
        &mut self,
        tx_queue: &mut Vec<j1939::Frame>,
        object: &Object,
    ) -> Result<(), J1939UnitError> {
        self.driver.trigger(&mut self.context, tx_queue, object)
    }

    fn teardown(&mut self, tx_queue: &mut Vec<j1939::Frame>) -> Result<(), J1939UnitError> {
        self.driver.teardown(&mut self.context, tx_queue)
    }
}

impl std::fmt::Display for NetDriverItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.driver.name())
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
                "[{}] Setup network driver: {}",
                self.network.interface(),
                driver
            );

            if let Err(e) = driver.setup(&mut tx_queue) {
                error!("[{}] {}: {}", self.network.interface(), driver, e);
            }

            if let Err(e) = self.network.send_vectored(&tx_queue).await {
                error!("[{}] {}: {}", self.network.interface(), driver, e);
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
                            self.network.interface(),
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                        rx_timeout: driver.rx_timeout,
                    });
                }
                ("laixer", "hcu") => {
                    drivers.push(NetDriverItem {
                        driver: Box::new(crate::driver::HydraulicControlUnit::new(
                            self.network.interface(),
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                        rx_timeout: driver.rx_timeout,
                    });
                }
                ("volvo", "d7e") => {
                    drivers.push(NetDriverItem {
                        driver: Box::new(crate::driver::VolvoD7E::new(
                            self.network.interface(),
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                        rx_timeout: driver.rx_timeout,
                    });
                }
                ("kübler", "inclinometer") => {
                    drivers.push(NetDriverItem {
                        driver: Box::new(crate::driver::KueblerInclinometer::new(
                            self.network.interface(),
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                        rx_timeout: driver.rx_timeout,
                    });
                }
                ("kübler", "encoder") => {
                    drivers.push(NetDriverItem {
                        driver: Box::new(crate::driver::KueblerEncoder::new(
                            self.network.interface(),
                            driver.driver.destination(),
                            driver.driver.source(),
                        )),
                        context: driver.context.clone(),
                        rx_timeout: driver.rx_timeout,
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

impl NetworkService<NetworkConfig> for NetworkAuthority {
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
                    let mut net_driver =
                        NetDriverItem::new(crate::driver::VehicleControlUnit::new(
                            network.interface(),
                            driver.da,
                            driver.sa.unwrap_or(config.address),
                        ));

                    if let Some(timeout) = driver.timeout {
                        net_driver.rx_timeout = Some(Duration::from_millis(timeout));
                    }

                    drivers.push(net_driver);
                }
                ("laixer", "hcu") => {
                    let mut net_driver =
                        NetDriverItem::new(crate::driver::HydraulicControlUnit::new(
                            network.interface(),
                            driver.da,
                            driver.sa.unwrap_or(config.address),
                        ));

                    if let Some(timeout) = driver.timeout {
                        net_driver.rx_timeout = Some(Duration::from_millis(timeout));
                    }

                    drivers.push(net_driver);
                }
                ("volvo", "d7e") => {
                    let mut net_driver = NetDriverItem::new(crate::driver::VolvoD7E::new(
                        network.interface(),
                        driver.da,
                        driver.sa.unwrap_or(config.address),
                    ));

                    if let Some(timeout) = driver.timeout {
                        net_driver.rx_timeout = Some(Duration::from_millis(timeout));
                    }

                    drivers.push(net_driver);
                }
                ("kübler", "inclinometer") => {
                    let mut net_driver =
                        NetDriverItem::new(crate::driver::KueblerInclinometer::new(
                            network.interface(),
                            driver.da,
                            driver.sa.unwrap_or(config.address),
                        ));

                    if let Some(timeout) = driver.timeout {
                        net_driver.rx_timeout = Some(Duration::from_millis(timeout));
                    }

                    drivers.push(net_driver);
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
                    let mut net_driver = NetDriverItem::new(crate::driver::KueblerEncoder::new(
                        network.interface(),
                        driver.da,
                        driver.sa.unwrap_or(config.address),
                    ));

                    if let Some(timeout) = driver.timeout {
                        net_driver.rx_timeout = Some(Duration::from_millis(timeout));
                    }

                    drivers.push(net_driver);
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

    async fn setup(&mut self) {
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
                    let timedate = j1939::spn::TimeDate::from_date_time(chrono::Utc::now());

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

        for driver in self.drivers.iter_mut() {
            match driver.try_recv(frame, signal_tx.clone()) {
                Ok(J1939UnitOk::SignalQueued) => {
                    driver.context.rx_mark();
                }
                Ok(J1939UnitOk::FrameParsed) => {
                    driver.context.rx_mark();
                }
                Ok(_) => {}
                Err(e) => {
                    error!("[{}] {}: {}", self.network.interface(), driver, e);
                }
            }
        }
    }

    async fn on_tick(&mut self) {
        for driver in self.drivers.iter_mut() {
            let mut tx_queue = Vec::new();

            if let Err(e) = driver.tick(&mut tx_queue) {
                error!("[{}] {}: {}", self.network.interface(), driver, e);
            }

            if driver.is_rx_timeout() {
                error!(
                    "[{}] {}: communication timeout",
                    self.network.interface(),
                    driver
                );
            }

            if let Err(e) = self.network.send_vectored(&tx_queue).await {
                error!("[{}] {}: {}", self.network.interface(), driver, e);
            };
        }
    }

    async fn on_command(&mut self, object: &Object) {
        for driver in self.drivers.iter_mut() {
            let mut tx_queue = Vec::new();

            if let Err(e) = driver.trigger(&mut tx_queue, object) {
                error!("[{}] {}: {}", self.network.interface(), driver, e);
            }

            if let Err(e) = self.network.send_vectored(&tx_queue).await {
                error!("[{}] {}: {}", self.network.interface(), driver, e);
            };
        }
    }

    async fn teardown(&mut self) {
        for driver in self.drivers.iter_mut() {
            let mut tx_queue = Vec::new();

            debug!(
                "[{}] Teardown network driver: {}",
                self.network.interface(),
                driver
            );

            if let Err(e) = driver.teardown(&mut tx_queue) {
                error!("[{}] {}: {}", self.network.interface(), driver, e);
            }

            if let Err(e) = self.network.send_vectored(&tx_queue).await {
                error!("[{}] {}: {}", self.network.interface(), driver, e);
            };
        }
    }
}
