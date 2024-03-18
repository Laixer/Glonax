use j1939::{Frame, FrameBuilder, IdBuilder, PGN};

use crate::net::Parsable;

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

#[derive(Debug, Clone)]
pub struct EncoderMessage {
    /// Source address.
    source_address: u8,
    /// Position.
    pub position: u32,
    /// Speed.
    pub speed: u16,
    /// State.
    pub state: Option<EncoderState>,
}

impl EncoderMessage {
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
            source_address: frame.id().sa(),
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

pub struct KueblerEncoder {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    _source_address: u8,
}

impl KueblerEncoder {
    /// Construct a new encoder service.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            _source_address: sa,
        }
    }
}

impl Parsable<EncoderMessage> for KueblerEncoder {
    fn parse(&mut self, frame: &Frame) -> Option<EncoderMessage> {
        if frame.id().pgn() == PGN::ProprietaryB(65_450) {
            if frame.id().sa() != self.destination_address {
                return None;
            }

            Some(EncoderMessage::from_frame(frame))
        } else {
            None
        }
    }
}

impl super::J1939Unit for KueblerEncoder {
    fn name(&self) -> &str {
        "Kubler encoder"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        match state {
            super::J1939UnitOperationState::Setup => {
                // log::debug!("[0x{:X}] Kubler encoder setup", self.destination_address);
            }
            super::J1939UnitOperationState::Running => {
                let mut result = Result::<(), super::J1939UnitError>::Ok(());

                if ctx.rx_last.elapsed().as_millis() > 1_000 {
                    result = Err(super::J1939UnitError::new(
                        "Kubler encoder".to_owned(),
                        self.destination_address,
                        super::J1939UnitErrorKind::MessageTimeout,
                    ));
                }

                if let Some(message) = router.try_accept(self) {
                    ctx.rx_last = std::time::Instant::now();

                    if let Ok(mut runtime_state) = runtime_state.try_write() {
                        runtime_state
                            .state
                            .encoders
                            .insert(message.source_address, message.position as f32);
                    }
                }

                result?
            }
            super::J1939UnitOperationState::Teardown => {
                // log::debug!("[0x{:X}] Kubler encoder teardown", self.destination_address);
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
        let message_a = EncoderMessage {
            source_address: 0x6A,
            position: 1_620,
            speed: 0,
            state: None,
        };

        let frames = message_a.to_frame();
        let messasge_b = EncoderMessage::from_frame(&frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(messasge_b.position, 1_620);
        assert_eq!(messasge_b.speed, 0);
        assert_eq!(messasge_b.state, Some(EncoderState::NoError));
    }

    #[test]
    fn value_error() {
        let messasge_a = EncoderMessage {
            source_address: 0x45,
            position: 173,
            speed: 65_196,
            state: Some(EncoderState::InvalidTMR),
        };

        let frames = messasge_a.to_frame();
        let messasge_b = EncoderMessage::from_frame(&frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(messasge_b.position, 173);
        assert_eq!(messasge_b.speed, 65_196);
        assert_eq!(messasge_b.state, Some(EncoderState::InvalidTMR));
    }
}
