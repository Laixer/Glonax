use j1939::{protocol, Frame, PGN};

use crate::net::Parsable;

#[derive(Debug, Clone)]
pub enum SensorOrientation {
    YPositive,
    YNegative,
    XPositive,
    XNegative,
}

// TODO: Missing a few fields.
#[derive(Debug, Clone)]
pub struct ProcessDataMessage {
    /// Source address.
    _source_address: u8,
    /// Slope long (Z-axis).
    pub slope_long: u16,
    /// Slope lat (X-axis).
    pub slope_lat: u16,
    /// Temperature.
    pub temperature: f32,
    /// Sensor is upside down.
    pub upside_down: bool,
    /// Sensor orientation.
    pub orientation: SensorOrientation,
}

impl ProcessDataMessage {
    /// Construct a new encoder message from a frame.
    pub fn from_frame(frame: &Frame) -> Self {
        let mut message = Self {
            _source_address: frame.id().source_address(),
            slope_long: 0,
            slope_lat: 0,
            temperature: 0.0,
            upside_down: false,
            orientation: SensorOrientation::YPositive,
        };

        let slope_long_bytes = &frame.pdu()[0..2];
        if slope_long_bytes != [0xff; 2] {
            message.slope_long = u16::from_le_bytes(slope_long_bytes.try_into().unwrap());
        };
        let slope_lat_bytes = &frame.pdu()[2..4];
        if slope_lat_bytes != [0xff; 2] {
            message.slope_lat = u16::from_le_bytes(slope_lat_bytes.try_into().unwrap());
        };

        let temperature_bytes = &frame.pdu()[4..6];
        if temperature_bytes != [0xff; 2] {
            let temperature = u16::from_le_bytes(temperature_bytes.try_into().unwrap());
            message.temperature = temperature as f32 / 10.0;
        };

        // let overflow = match frame.pdu()[6] & 0b11 {
        //     0 => false,
        //     1 => true,
        //     _ => unimplemented!(),
        // }
        message.upside_down = match frame.pdu()[6] >> 2 & 0b11 {
            0 => false,
            1 => true,
            _ => unimplemented!(),
        };
        let error = match frame.pdu()[6] >> 4 {
            0 => false,
            0xE => true, // Invalid configuration
            0xD => true, // Reading error when accessing to the internal sensor components.
            _ => unimplemented!(),
        };

        if frame.pdu()[7] != 0xff {
            message.orientation = match frame.pdu()[7] {
                0 => SensorOrientation::YPositive,
                2 => SensorOrientation::YNegative,
                4 => SensorOrientation::XPositive,
                6 => SensorOrientation::XNegative,
                _ => unimplemented!(),
            };
        }

        message
    }
}

impl std::fmt::Display for ProcessDataMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Slope Long: {:>5}; Slope Lat: {:>5}; Temperature: {:>5}",
            self.slope_long, self.slope_lat, self.temperature
        )
    }
}

pub struct KueblerInclinometer {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
}

impl KueblerInclinometer {
    /// Construct a new encoder service.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
        }
    }
}

impl Parsable<ProcessDataMessage> for KueblerInclinometer {
    fn parse(&mut self, frame: &Frame) -> Option<ProcessDataMessage> {
        if let Some(destination_address) = frame.id().destination_address() {
            if destination_address != self.destination_address && destination_address != 0xff {
                return None;
            }
        }

        if frame.id().pgn() == PGN::ProprietaryB(65_451) {
            if frame.id().source_address() != self.destination_address {
                return None;
            }

            Some(ProcessDataMessage::from_frame(frame))
        } else {
            None
        }
    }
}

impl super::J1939Unit for KueblerInclinometer {
    fn name(&self) -> &str {
        "Kubler inclinometer"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    #[rustfmt::skip]
    async fn setup(
        &self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        network.send(&protocol::request(self.destination_address, self.source_address, PGN::AddressClaimed)).await?;
        network.send(&protocol::request(self.destination_address, self.source_address, PGN::SoftwareIdentification)).await?;
        network.send(&protocol::request(self.destination_address, self.source_address, PGN::ComponentIdentification)).await?;
        ctx.tx_mark();

        Ok(())
    }

    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        let mut result = Result::<(), super::J1939UnitError>::Ok(());

        if ctx.is_rx_timeout(std::time::Duration::from_millis(1_000)) {
            result = Err(super::J1939UnitError::MessageTimeout);
        }

        if let Some(_message) = network.try_accept(self) {
            ctx.rx_mark();
        }

        result
    }
}
