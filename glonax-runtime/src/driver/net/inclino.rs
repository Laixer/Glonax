use j1939::{protocol, Frame, Name, PGN};

use crate::net::Parsable;

const _CONFIG_PGN: PGN = PGN::ProprietaryA;
const INCLINOMETER_PGN: PGN = PGN::ProprietaryB(65_451);

// TODO: Add configuration message.

#[derive(Debug, Clone, PartialEq)]
pub enum InclinometerStatus {
    /// No error.
    NoError,
    /// Invalid configuration.
    InvalidConfiguration,
    /// General error in sensor.
    GeneralSensorError,
    /// Unknown error.
    Other,
}

impl std::fmt::Display for InclinometerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InclinometerStatus::NoError => write!(f, "no error"),
            InclinometerStatus::InvalidConfiguration => write!(f, "invalid configuration"),
            InclinometerStatus::GeneralSensorError => write!(f, "general error in sensor"),
            InclinometerStatus::Other => write!(f, "unknown error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Overflow {
    ValidRange,
    OutsidePositiveRange,
    OutsideNegativeRange,
    Other,
}

#[derive(Debug, Clone)]
pub enum SensorOrientation {
    YPositive,
    YNegative,
    XPositive,
    XNegative,
    Other,
}

pub enum InclinoMessage {
    ProcessData(ProcessDataMessage),
    AddressClaim(Name),
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
    /// Sensor overflow.
    pub overflow: Overflow,
    /// Sensor orientation.
    pub orientation: SensorOrientation,
    /// Sensor status.
    pub status: InclinometerStatus,
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
            overflow: Overflow::ValidRange,
            orientation: SensorOrientation::YPositive,
            status: InclinometerStatus::NoError,
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

        if frame.pdu()[6] != 0xff {
            message.overflow = match frame.pdu()[6] & 0b11 {
                0 => Overflow::ValidRange,
                1 => Overflow::OutsidePositiveRange,
                2 => Overflow::OutsideNegativeRange,
                _ => Overflow::Other,
            };

            message.upside_down = match frame.pdu()[6] >> 2 & 0b11 {
                0 => false,
                1 => true,
                _ => false,
            };

            message.status = match frame.pdu()[6] >> 4 {
                0x0 => InclinometerStatus::NoError,
                0xe => InclinometerStatus::InvalidConfiguration,
                0xed => InclinometerStatus::GeneralSensorError,
                _ => InclinometerStatus::Other,
            };
        }

        if frame.pdu()[7] != 0xff {
            message.orientation = match frame.pdu()[7] {
                0 => SensorOrientation::YPositive,
                2 => SensorOrientation::YNegative,
                4 => SensorOrientation::XPositive,
                6 => SensorOrientation::XNegative,
                _ => SensorOrientation::Other,
            };
        }

        message
    }
}

impl std::fmt::Display for ProcessDataMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Slope Long: {:>5}; Slope Lat: {:>5}; Temperature: {:>5.2}°; Overflow: {:?}; Orientation: {:?}; Status: {}",
            self.slope_long, self.slope_lat, self.temperature, self.overflow, self.orientation, self.status
        )
    }
}

#[derive(Clone)]
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

impl Parsable<InclinoMessage> for KueblerInclinometer {
    fn parse(&self, frame: &Frame) -> Option<InclinoMessage> {
        if let Some(destination_address) = frame.id().destination_address() {
            if destination_address != self.destination_address && destination_address != 0xff {
                return None;
            }
        }

        match frame.id().pgn() {
            PGN::AddressClaimed => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(InclinoMessage::AddressClaim(Name::from_bytes(
                    frame.pdu().try_into().unwrap(),
                )))
            }
            INCLINOMETER_PGN => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(InclinoMessage::ProcessData(ProcessDataMessage::from_frame(
                    frame,
                )))
            }
            _ => None,
        }
    }
}

impl super::J1939Unit for KueblerInclinometer {
    fn vendor(&self) -> &'static str {
        "kübler"
    }

    fn product(&self) -> &'static str {
        "inclinometer"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    fn setup(
        &self,
        _ctx: &mut super::NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), super::J1939UnitError> {
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::AddressClaimed,
        ));

        Ok(())
    }

    fn try_recv(
        &self,
        _ctx: &mut super::NetDriverContext,
        frame: &j1939::Frame,
        _signal_tx: crate::runtime::SignalSender,
    ) -> Result<super::J1939UnitOk, super::J1939UnitError> {
        if let Some(message) = self.parse(frame) {
            match message {
                InclinoMessage::AddressClaim(name) => {
                    debug!(
                        "[{}] {}:0x{:X}: Address claimed: {}",
                        // network.interface(),
                        "kaas0",
                        self.name(),
                        self.destination(),
                        name
                    );

                    return Ok(super::J1939UnitOk::FrameParsed);
                }
                InclinoMessage::ProcessData(_process_data) => {
                    return Ok(super::J1939UnitOk::FrameParsed);
                }
            }
        }

        Ok(super::J1939UnitOk::FrameIgnored)
    }
}
