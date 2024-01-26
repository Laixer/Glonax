use glonax_j1939::{
    decode::{EngineStarterMode, EngineTorqueMode},
    *,
};

use crate::net::Parsable;

#[derive(Default)]
pub struct EngineMessage {
    /// Engine Torque Mode.
    pub engine_torque_mode: Option<EngineTorqueMode>,
    /// Driver's Demand Engine - Percent Torque.
    pub driver_demand: Option<u8>,
    /// Actual Engine - Percent Torque.
    pub actual_engine: Option<u8>,
    /// Engine Speed.
    pub rpm: Option<u16>,
    /// Source Address of Controlling Device for Engine Control.
    pub source_addr: Option<u8>,
    /// Engine Starter Mode.
    pub starter_mode: Option<EngineStarterMode>,
}

impl EngineMessage {
    pub fn from_frame(frame: &Frame) -> Self {
        Self {
            engine_torque_mode: decode::spn899(frame.pdu()[0]),
            driver_demand: decode::spn512(frame.pdu()[1]),
            actual_engine: decode::spn513(frame.pdu()[2]),
            rpm: decode::spn190(&frame.pdu()[3..5].try_into().unwrap()),
            source_addr: decode::spn1483(frame.pdu()[5]),
            starter_mode: decode::spn1675(frame.pdu()[6]),
        }
    }

    #[allow(dead_code)]
    fn to_frame(&self) -> Vec<Frame> {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ElectronicEngineController1)
                .da(0xff)
                .build(),
        );

        if let Some(driver_demand) = self.driver_demand {
            frame_builder.as_mut()[1] = driver_demand + 125;
        }
        if let Some(actual_engine) = self.actual_engine {
            frame_builder.as_mut()[2] = actual_engine + 125;
        }

        if let Some(rpm) = self.rpm {
            let rpm_bytes = (rpm * 8).to_le_bytes();
            frame_builder.as_mut()[3..5].copy_from_slice(&rpm_bytes);
        }

        if let Some(source_addr) = self.source_addr {
            frame_builder.as_mut()[5] = source_addr;
        }

        vec![frame_builder.set_len(8).build()]
    }
}

impl std::fmt::Display for EngineMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        if let Some(engine_torque_mode) = &self.engine_torque_mode {
            s.push_str(&format!("Torque mode: {:?}; ", engine_torque_mode));
        }

        if let Some(driver_demand) = self.driver_demand {
            s.push_str(&format!("Driver Demand: {}%; ", driver_demand));
        }

        if let Some(actual_engine) = self.actual_engine {
            s.push_str(&format!("Actual Engine: {}%; ", actual_engine));
        }

        if let Some(rpm) = self.rpm {
            s.push_str(&format!("RPM: {}; ", rpm));
        }

        if let Some(starter_mode) = &self.starter_mode {
            s.push_str(&format!("Starter mode: {:?}; ", starter_mode));
        }

        write!(f, "{}", s)
    }
}

const VOLVO_VECU_J1939_ADDRESS: u8 = 0x11;

#[derive(Default)]
pub struct EngineManagementSystem;

impl EngineManagementSystem {
    pub fn set_rpm(&self, rpm: u16) -> Vec<Frame> {
        #[allow(dead_code)]
        enum EngineMode {
            /// Engine shutdown.
            Shutdown = 0x07,
            /// Engine starter locked.
            Locked = 0x47,
            /// Engine running at requested speed.
            Nominal = 0x43,
            /// Engine starter engaged.
            Starting = 0xC3,
        }

        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietaryB(65_282))
                .priority(3)
                .sa(VOLVO_VECU_J1939_ADDRESS)
                .build(),
        )
        .copy_from_slice(&[
            0x00,
            EngineMode::Nominal as u8,
            0x1f,
            0x00,
            0x00,
            0x00,
            0x20,
            (rpm as f32 / 10.0) as u8,
        ])
        .build();

        vec![frame]
    }
}

impl Parsable<EngineMessage> for EngineManagementSystem {
    fn parse(&mut self, frame: &Frame) -> Option<EngineMessage> {
        if frame.len() != 8 {
            return None;
        }
        if frame.id().pgn() != PGN::ElectronicEngineController1 {
            return None;
        }

        Some(EngineMessage::from_frame(frame))
    }
}

impl super::J1939Unit for EngineManagementSystem {
    fn try_accept(
        &mut self,
        router: &mut crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        if let Some(message) = router.try_accept(self) {
            if let Ok(mut runtime_state) = runtime_state.try_write() {
                runtime_state.state.engine.driver_demand = message.driver_demand.unwrap_or(0);
                runtime_state.state.engine.actual_engine = message.actual_engine.unwrap_or(0);
                runtime_state.state.engine.rpm = message.rpm.unwrap_or(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_on() {
        let frame =
            FrameBuilder::new(IdBuilder::from_pgn(PGN::ElectronicEngineController1).build())
                .copy_from_slice(&[0xF0, 0xEA, 0x7D, 0x00, 0x00, 0x00, 0xF0, 0xFF])
                .build();

        let engine_message = EngineMessage::from_frame(&frame);
        assert_eq!(
            engine_message.engine_torque_mode.unwrap(),
            EngineTorqueMode::NoRequest
        );
        assert_eq!(engine_message.driver_demand.unwrap(), 109);
        assert_eq!(engine_message.actual_engine.unwrap(), 0);
        assert_eq!(engine_message.rpm.unwrap(), 0);
        assert_eq!(engine_message.source_addr.unwrap(), 0);
        assert_eq!(
            engine_message.starter_mode.unwrap(),
            EngineStarterMode::StartNotRequested
        );
    }

    #[test]
    fn turn_off() {
        let frame =
            FrameBuilder::new(IdBuilder::from_pgn(PGN::ElectronicEngineController1).build())
                .copy_from_slice(&[0xF3, 0x91, 0x91, 0xAA, 0x18, 0x00, 0xF3, 0xFF])
                .build();

        let engine_message = EngineMessage::from_frame(&frame);
        assert_eq!(
            engine_message.engine_torque_mode.unwrap(),
            EngineTorqueMode::PTOGovernor
        );
        assert_eq!(engine_message.driver_demand.unwrap(), 20);
        assert_eq!(engine_message.actual_engine.unwrap(), 20);
        assert_eq!(engine_message.rpm.unwrap(), 789);
        assert_eq!(engine_message.source_addr.unwrap(), 0);
        assert_eq!(
            engine_message.starter_mode.unwrap(),
            EngineStarterMode::StartFinished
        );
    }
}
