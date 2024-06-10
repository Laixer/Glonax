use j1939::{protocol, Frame, FrameBuilder, IdBuilder, Name, PGN};

use crate::{
    core::{Object, ObjectMessage, Rotator},
    driver::EncoderConverter,
    net::Parsable,
    runtime::{J1939Unit, J1939UnitError, NetDriverContext},
};

const _CONFIG_PGN: PGN = PGN::ProprietaryA;
const ENCODER_PGN: PGN = PGN::ProprietaryB(65_450);

// TODO: Add configuration message.

// TODO: Should this be EncoderStatus?
#[derive(Debug, Clone, PartialEq)]
pub enum EncoderState {
    /// No error.
    NoError,
    /// General error in sensor.
    GeneralSensorError,
    /// Invalid MUR value.
    InvalidMUR,
    /// Invalid TMR value.
    InvalidTMR,
    /// Invalid preset value.
    InvalidPreset,
    /// Unknown error.
    Other,
}

impl std::fmt::Display for EncoderState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncoderState::NoError => write!(f, "no error"),
            EncoderState::GeneralSensorError => write!(f, "general error in sensor"),
            EncoderState::InvalidMUR => write!(f, "invalid MUR value"),
            EncoderState::InvalidTMR => write!(f, "invalid TMR value"),
            EncoderState::InvalidPreset => write!(f, "invalid preset value"),
            EncoderState::Other => write!(f, "unknown error"),
        }
    }
}

pub enum EncoderMessage {
    ProcessData(ProcessDataMessage),
    AddressClaim(Name),
}

// TODO: Every message should implement a trait that allows it to be converted to and from a frame.
// TODO: Implement from_frame and to_frame for EncoderMessage.
// TODO: Implement Display for message traits.
#[derive(Debug, Clone)]
pub struct ProcessDataMessage {
    /// Source address.
    source_address: u8,
    /// Position.
    position: u32,
    /// Speed.
    speed: u16,
    /// State.
    state: EncoderState,
}

impl ProcessDataMessage {
    /// Construct a new encoder message.
    pub fn new(sa: u8) -> Self {
        Self {
            source_address: sa,
            position: 0,
            speed: 0,
            state: EncoderState::NoError,
        }
    }

    /// Construct a new encoder message with position.
    pub fn from_position(sa: u8, position: u32) -> Self {
        Self {
            source_address: sa,
            position,
            speed: 0,
            state: EncoderState::NoError,
        }
    }

    /// Construct a new encoder message from a frame.
    pub fn from_frame(frame: &Frame) -> Self {
        let mut message = Self {
            source_address: frame.id().source_address(),
            position: 0,
            speed: 0,
            state: EncoderState::NoError,
        };

        let position_bytes = &frame.pdu()[0..4];
        if position_bytes != [0xff; 4] {
            message.position = u32::from_le_bytes(position_bytes.try_into().unwrap());
        };

        let speed_bytes = &frame.pdu()[4..6];
        if speed_bytes != [0xff; 2] {
            message.speed = u16::from_le_bytes(speed_bytes.try_into().unwrap());
        };

        let state_bytes = &frame.pdu()[6..8];
        if state_bytes != [0xff; 2] {
            let state = u16::from_le_bytes(state_bytes.try_into().unwrap());

            message.state = match state {
                0x0 => EncoderState::NoError,
                0xee00 => EncoderState::GeneralSensorError,
                0xee01 => EncoderState::InvalidMUR,
                0xee02 => EncoderState::InvalidTMR,
                0xee03 => EncoderState::InvalidPreset,
                _ => EncoderState::Other,
            };
        }

        message
    }

    #[allow(dead_code)]
    fn to_frame(&self) -> Vec<Frame> {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietaryB(65_450))
                .sa(self.source_address)
                .build(),
        );

        let position_bytes = self.position.to_le_bytes();
        frame_builder.as_mut()[0..4].copy_from_slice(&position_bytes);

        let speed_bytes = self.speed.to_le_bytes();
        frame_builder.as_mut()[4..6].copy_from_slice(&speed_bytes);

        let state_bytes = match self.state {
            EncoderState::NoError => 0x0_u16,
            EncoderState::GeneralSensorError => 0xee00,
            EncoderState::InvalidMUR => 0xee01,
            EncoderState::InvalidTMR => 0xee02,
            EncoderState::InvalidPreset => 0xee03,
            EncoderState::Other => 0xeeff,
        }
        .to_le_bytes();
        frame_builder.as_mut()[6..8].copy_from_slice(&state_bytes);

        vec![frame_builder.set_len(8).build()]
    }
}

impl From<Frame> for ProcessDataMessage {
    fn from(frame: Frame) -> Self {
        Self::from_frame(&frame)
    }
}

impl From<&Frame> for ProcessDataMessage {
    fn from(frame: &Frame) -> Self {
        Self::from_frame(frame)
    }
}

impl std::fmt::Display for ProcessDataMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Position: {:>5} {:>6.2}rad {:>6.2}°; Speed {:>5}; State: {}",
            self.position,
            self.position as f32 / 1000.0,
            (self.position as f32 / 1000.0).to_degrees(),
            self.speed,
            self.state,
        )
    }
}

#[derive(Clone)]
pub struct KueblerEncoder {
    /// Network interface.
    interface: String,
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
    /// Converter.
    converter: EncoderConverter,
}

impl KueblerEncoder {
    /// Construct a new encoder service.
    pub fn new(interface: &str, da: u8, sa: u8) -> Self {
        let converter = if da == 0x6a {
            EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::z_axis())
        } else if da == 0x6b {
            EncoderConverter::new(
                1000.0,
                60_f32.to_radians(),
                true,
                nalgebra::Vector3::y_axis(),
            )
        } else if da == 0x6c || da == 0x6d {
            EncoderConverter::new(1000.0, 0.0, true, nalgebra::Vector3::y_axis())
        } else {
            panic!("Unknown encoder address: {:x}", da)
        };

        Self {
            interface: interface.to_string(),
            destination_address: da,
            source_address: sa,
            converter,
        }
    }
}

impl Parsable<EncoderMessage> for KueblerEncoder {
    fn parse(&self, frame: &Frame) -> Option<EncoderMessage> {
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

                Some(EncoderMessage::AddressClaim(Name::from_bytes(
                    frame.pdu().try_into().unwrap(),
                )))
            }
            ENCODER_PGN => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(EncoderMessage::ProcessData(ProcessDataMessage::from_frame(
                    frame,
                )))
            }
            _ => None,
        }
    }
}

impl J1939Unit for KueblerEncoder {
    fn vendor(&self) -> &'static str {
        "kübler"
    }

    fn product(&self) -> &'static str {
        "encoder"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    fn setup(
        &self,
        _ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::AddressClaimed,
        ));

        Ok(())
    }

    fn try_recv(
        &self,
        ctx: &mut NetDriverContext,
        frame: &j1939::Frame,
        rx_queue: &mut Vec<Object>,
    ) -> Result<(), J1939UnitError> {
        if let Some(message) = self.parse(frame) {
            match message {
                EncoderMessage::AddressClaim(name) => {
                    debug!(
                        "[{}] {}: Address claimed: {}",
                        self.interface,
                        self.name(),
                        name
                    );

                    return Ok(());
                }
                EncoderMessage::ProcessData(process_data) => {
                    let rotation = self.converter.to_rotation(process_data.position as f32);
                    let rotator = Rotator::relative(process_data.source_address, rotation);

                    trace!(
                        "[{}] {}: Roll={:.2} Pitch={:.2} Yaw={:.2}",
                        self.interface,
                        self.name(),
                        rotation.euler_angles().0.to_degrees(),
                        rotation.euler_angles().1.to_degrees(),
                        rotation.euler_angles().2.to_degrees()
                    );

                    ctx.set_rx_last_message(ObjectMessage::signal(Object::Rotator(rotator)));

                    rx_queue.push(Object::Rotator(rotator));

                    return match process_data.state {
                        EncoderState::GeneralSensorError => Err(J1939UnitError::SensorError),
                        EncoderState::InvalidMUR => Err(J1939UnitError::InvalidConfiguration),
                        EncoderState::InvalidTMR => Err(J1939UnitError::InvalidConfiguration),
                        EncoderState::InvalidPreset => Err(J1939UnitError::InvalidConfiguration),
                        EncoderState::Other => Err(J1939UnitError::HardwareError),
                        EncoderState::NoError => Ok(()),
                    };
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_normal() {
        let message_a = ProcessDataMessage {
            source_address: 0x6A,
            position: 1_620,
            speed: 0,
            state: EncoderState::NoError,
        };

        let frames = message_a.to_frame();
        let messasge_b = ProcessDataMessage::from_frame(&frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(messasge_b.position, 1_620);
        assert_eq!(messasge_b.speed, 0);
        assert_eq!(messasge_b.state, EncoderState::NoError);
    }

    #[test]
    fn value_error() {
        let messasge_a = ProcessDataMessage {
            source_address: 0x45,
            position: 173,
            speed: 65_196,
            state: EncoderState::InvalidTMR,
        };

        let frames = messasge_a.to_frame();
        let messasge_b = ProcessDataMessage::from_frame(&frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(messasge_b.position, 173);
        assert_eq!(messasge_b.speed, 65_196);
        assert_eq!(messasge_b.state, EncoderState::InvalidTMR);
    }
}
