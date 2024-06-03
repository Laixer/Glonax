use j1939::{protocol, Frame, FrameBuilder, IdBuilder, Name, PGN};

use crate::{
    core::{Object, ObjectMessage},
    net::Parsable,
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
    pub position: u32,
    /// Speed.
    pub speed: u16,
    /// State.
    pub state: Option<EncoderState>,
}

impl ProcessDataMessage {
    /// Construct a new encoder message.
    pub fn new(sa: u8) -> Self {
        Self {
            source_address: sa,
            position: 0,
            speed: 0,
            state: None,
        }
    }

    /// Construct a new encoder message with position.
    pub fn from_position(sa: u8, position: u32) -> Self {
        Self {
            source_address: sa,
            position,
            speed: 0,
            state: None,
        }
    }

    /// Construct a new encoder message from a frame.
    pub fn from_frame(frame: &Frame) -> Self {
        let mut message = Self {
            source_address: frame.id().source_address(),
            position: 0,
            speed: 0,
            state: None,
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

            message.state = Some(match state {
                0x0 => EncoderState::NoError,
                0xee00 => EncoderState::GeneralSensorError,
                0xee01 => EncoderState::InvalidMUR,
                0xee02 => EncoderState::InvalidTMR,
                0xee03 => EncoderState::InvalidPreset,
                _ => EncoderState::Other,
            });
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
            Some(EncoderState::NoError) => 0x0,
            Some(EncoderState::GeneralSensorError) => 0xee00,
            Some(EncoderState::InvalidMUR) => 0xee01,
            Some(EncoderState::InvalidTMR) => 0xee02,
            Some(EncoderState::InvalidPreset) => 0xee03,
            Some(EncoderState::Other) => 0xeeff,
            None => 0x0_u16,
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
            self.state
                .as_ref()
                .map_or_else(|| "-".to_owned(), |f| f.to_string()),
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
}

impl KueblerEncoder {
    /// Construct a new encoder service.
    pub fn new(interface: &str, da: u8, sa: u8) -> Self {
        Self {
            interface: interface.to_string(),
            destination_address: da,
            source_address: sa,
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

impl super::J1939Unit for KueblerEncoder {
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
        ctx: &mut super::NetDriverContext,
        frame: &j1939::Frame,
        signal_tx: crate::runtime::SignalSender,
    ) -> Result<super::J1939UnitOk, super::J1939UnitError> {
        if let Some(message) = self.parse(frame) {
            match message {
                EncoderMessage::AddressClaim(name) => {
                    debug!(
                        "[{}] {}: Address claimed: {}",
                        self.interface,
                        self.name(),
                        name
                    );

                    return Ok(super::J1939UnitOk::FrameParsed);
                }
                EncoderMessage::ProcessData(process_data) => {
                    let encoder_signal =
                        (process_data.source_address, process_data.position as f32);

                    ctx.set_rx_last_message(ObjectMessage::signal(Object::Encoder(encoder_signal)));

                    if let Err(e) = signal_tx.send(Object::Encoder(encoder_signal)) {
                        error!("Failed to send encoder signal: {}", e);
                    }

                    return Ok(super::J1939UnitOk::SignalQueued);
                }
            }
        }

        Ok(super::J1939UnitOk::FrameIgnored)
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
            state: None,
        };

        let frames = message_a.to_frame();
        let messasge_b = ProcessDataMessage::from_frame(&frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(messasge_b.position, 1_620);
        assert_eq!(messasge_b.speed, 0);
        assert_eq!(messasge_b.state, Some(EncoderState::NoError));
    }

    #[test]
    fn value_error() {
        let messasge_a = ProcessDataMessage {
            source_address: 0x45,
            position: 173,
            speed: 65_196,
            state: Some(EncoderState::InvalidTMR),
        };

        let frames = messasge_a.to_frame();
        let messasge_b = ProcessDataMessage::from_frame(&frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(messasge_b.position, 173);
        assert_eq!(messasge_b.speed, 65_196);
        assert_eq!(messasge_b.state, Some(EncoderState::InvalidTMR));
    }
}
