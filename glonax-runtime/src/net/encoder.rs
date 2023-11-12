use glonax_j1939::*;

use crate::SharedRuntimeState;

use super::Parsable;

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

pub struct EncoderService {
    /// Node ID.
    node: u8,
}

#[derive(Debug, Clone)]
pub struct EncoderMessage {
    /// Node ID.
    pub node: u8,
    /// Position.
    pub position: u32,
    /// Speed.
    pub speed: u16,
    /// State.
    pub state: Option<EncoderState>,
}

impl EncoderMessage {
    /// Construct a new encoder message.
    pub fn new(node: u8) -> Self {
        Self {
            node,
            position: 0,
            speed: 0,
            state: None,
        }
    }

    /// Construct a new encoder message with position.
    pub fn new_with_position(node: u8, position: u32) -> Self {
        Self {
            node,
            position,
            speed: 0,
            state: None,
        }
    }

    pub fn from_frame(node: u8, frame: &Frame) -> Self {
        let mut this = Self {
            node,
            position: 0,
            speed: 0,
            state: None,
        };

        let position_bytes = &frame.pdu()[0..4];
        if position_bytes != [0xff; 4] {
            this.position = u32::from_le_bytes(position_bytes.try_into().unwrap());
        };

        let speed_bytes = &frame.pdu()[4..6];
        if speed_bytes != [0xff; 2] {
            this.speed = u16::from_le_bytes(speed_bytes.try_into().unwrap());
        };

        let state_bytes = &frame.pdu()[6..8];
        if state_bytes != [0xff; 2] {
            let state = u16::from_le_bytes(state_bytes.try_into().unwrap());

            this.state = Some(match state {
                0x0 => EncoderState::NoError,
                0xee00 => EncoderState::GeneralSensorError,
                0xee01 => EncoderState::InvalidMUR,
                0xee02 => EncoderState::InvalidTMR,
                0xee03 => EncoderState::InvalidPreset,
                _ => EncoderState::Other,
            });
        }

        this
    }

    fn to_frame(&self) -> Vec<Frame> {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietaryB(65_450))
                .sa(self.node)
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

    pub async fn fill(&self, local_runtime_state: SharedRuntimeState) {
        local_runtime_state
            .write()
            .await
            .state
            .pose
            .set_node_position(self.node, self.position);
    }
}

impl std::fmt::Display for EncoderMessage {
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

impl EncoderService {
    pub fn new(node: u8) -> Self {
        Self { node }
    }

    // TODO: Maybe not
    pub fn encode(&self, position: u32, speed: u16) -> Vec<Frame> {
        EncoderMessage {
            node: self.node,
            position,
            speed,
            state: None,
        }
        .to_frame()
    }
}

impl Parsable<EncoderMessage> for EncoderService {
    fn parse(&mut self, frame: &Frame) -> Option<EncoderMessage> {
        if frame.len() != 8 {
            return None;
        }
        if frame.id().pgn() != PGN::ProprietaryB(65_450) {
            return None;
        }
        if frame.id().sa() != self.node {
            return None;
        }

        Some(EncoderMessage::from_frame(self.node, frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_normal() {
        let message_a = EncoderMessage {
            node: 0x6A,
            position: 1_620,
            speed: 0,
            state: None,
        };

        let frames = message_a.to_frame();
        let messasge_b = EncoderMessage::from_frame(0x6A, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(messasge_b.position, 1_620);
        assert_eq!(messasge_b.speed, 0);
        assert_eq!(messasge_b.state.unwrap(), EncoderState::NoError);
    }

    #[test]
    fn value_error() {
        let messasge_a = EncoderMessage {
            node: 0x45,
            position: 173,
            speed: 65_196,
            state: Some(EncoderState::InvalidTMR),
        };

        let frames = messasge_a.to_frame();
        let messasge_b = EncoderMessage::from_frame(0x45, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(messasge_b.position, 173);
        assert_eq!(messasge_b.speed, 65_196);
        assert_eq!(messasge_b.state.unwrap(), EncoderState::InvalidTMR);
    }
}
