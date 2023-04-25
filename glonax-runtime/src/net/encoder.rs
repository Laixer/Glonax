use glonax_j1939::*;

use super::Routable;

#[derive(Debug, PartialEq)]
pub enum EncoderState {
    NoError,
    GeneralSensorError,
    InvalidMUR,
    InvalidTMR,
    InvalidPreset,
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

pub struct KueblerEncoderService {
    /// Node ID.
    pub node: u8,
    /// Position.
    position: u32,
    /// Speed.
    speed: u16,
    /// State.
    state: Option<EncoderState>,
}

impl Routable for KueblerEncoderService {
    fn ingress(&mut self, frame: &Frame) -> bool {
        if frame.id().pgn() != PGN::ProprietaryB(65_450) {
            return false;
        }
        if frame.id().sa() != self.node {
            return false;
        }

        let position_bytes = &frame.pdu()[0..4];
        if position_bytes != [0xff; 4] {
            self.position = u32::from_le_bytes(position_bytes.try_into().unwrap());
        };

        let speed_bytes = &frame.pdu()[4..6];
        if speed_bytes != [0xff; 2] {
            self.speed = u16::from_le_bytes(speed_bytes.try_into().unwrap());
        };

        let state_bytes = &frame.pdu()[6..8];
        if state_bytes != [0xff; 2] {
            let state = u16::from_le_bytes(state_bytes.try_into().unwrap());

            self.state = Some(match state {
                0x0 => EncoderState::NoError,
                0xee00 => EncoderState::GeneralSensorError,
                0xee01 => EncoderState::InvalidMUR,
                0xee02 => EncoderState::InvalidTMR,
                0xee03 => EncoderState::InvalidPreset,
                _ => EncoderState::Other,
            });
        }

        true
    }

    fn encode(&self) -> Vec<Frame> {
        let mut frames = Vec::new();

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

        frames.push(frame_builder.set_len(8).build());

        frames
    }
}

impl std::fmt::Display for KueblerEncoderService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Position: {:>5} {:>6.2}rad {:>6.2}°; Speed {:>5}; State: {}",
            self.position,
            self.position as f32 / 1000.0,
            crate::core::rad_to_deg(self.position as f32 / 1000.0),
            self.speed,
            self.state
                .as_ref()
                .map_or_else(|| "-".to_owned(), |f| f.to_string()),
        )
    }
}

impl crate::channel::BroadcastSource<crate::transport::Signal> for KueblerEncoderService {
    fn fetch(&self, writer: &crate::channel::BroadcastChannelWriter<crate::transport::Signal>) {
        writer.send(crate::transport::Signal::new(
            self.node as u32,
            0,
            crate::transport::signal::Metric::Angle(self.position as f32 / 1000.0),
        ));
        writer.send(crate::transport::Signal::new(
            self.node as u32,
            1,
            crate::transport::signal::Metric::Rpm(self.speed as i32),
        ));
    }
}

impl KueblerEncoderService {
    pub fn new(node: u8) -> Self {
        Self {
            node,
            position: 0,
            speed: 0,
            state: None,
        }
    }

    /// Create a new encoder service with a known position.
    pub fn with_value(node: u8, position: u32, speed: u16, state: EncoderState) -> Self {
        Self {
            node,
            position,
            speed,
            state: Some(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoder_value_normal() {
        let encoder_a = KueblerEncoderService::with_value(0x6A, 1_620, 0, EncoderState::NoError);
        let mut encoder_b = KueblerEncoderService::new(0x6A);

        let frames = encoder_a.encode();

        assert_eq!(frames.len(), 1);
        assert_eq!(encoder_b.ingress(&frames[0]), true);
        assert_eq!(encoder_b.position, 1_620);
        assert_eq!(encoder_b.speed, 0);
        assert_eq!(encoder_b.state.unwrap(), EncoderState::NoError);
    }

    #[test]
    fn encoder_value_error() {
        let encoder_a = KueblerEncoderService::with_value(0x45, 173, 65_196, EncoderState::InvalidTMR);
        let mut encoder_b = KueblerEncoderService::new(0x45);

        let frames = encoder_a.encode();

        assert_eq!(frames.len(), 1);
        assert_eq!(encoder_b.ingress(&frames[0]), true);
        assert_eq!(encoder_b.position, 173);
        assert_eq!(encoder_b.speed, 65_196);
        assert_eq!(encoder_b.state.unwrap(), EncoderState::InvalidTMR);
    }
}
