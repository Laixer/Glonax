use j1939::{Frame, PGN};

use crate::net::Parsable;

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
    pub temperature: u16,
}

impl ProcessDataMessage {
    /// Construct a new encoder message from a frame.
    pub fn from_frame(frame: &Frame) -> Self {
        let mut message = Self {
            _source_address: frame.id().sa(),
            slope_long: 0,
            slope_lat: 0,
            temperature: 0,
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
            message.temperature = u16::from_le_bytes(temperature_bytes.try_into().unwrap());
        };

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
    _source_address: u8,
}

impl KueblerInclinometer {
    /// Construct a new encoder service.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            _source_address: sa,
        }
    }
}

impl Parsable<ProcessDataMessage> for KueblerInclinometer {
    fn parse(&mut self, frame: &Frame) -> Option<ProcessDataMessage> {
        if frame.id().pgn() == PGN::ProprietaryB(65_451) {
            if frame.id().sa() != self.destination_address {
                return None;
            }

            Some(ProcessDataMessage::from_frame(frame))
        } else {
            None
        }
    }
}

impl super::J1939Unit for KueblerInclinometer {
    async fn try_accept(
        &mut self,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        if let Some(_message) = router.try_accept(self) {
            if let Ok(mut _runtime_state) = runtime_state.try_write() {
                // runtime_state
                //     .state
                //     .encoders
                //     .insert(message.source_address, message.position as f32);
            }
        }
    }
}
