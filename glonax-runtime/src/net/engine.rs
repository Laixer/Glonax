use glonax_j1939::{
    decode::{EngineStarterMode, EngineTorqueMode},
    *,
};

use crate::{runtime::SharedOperandState, RobotState};

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

    fn to_frame(&self) -> Vec<Frame> {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ElectronicEngineController2)
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

    pub async fn fill<R: RobotState>(&self, local_runtime_state: SharedOperandState<R>) {
        let mut runtime_state = local_runtime_state.write().await;

        runtime_state.state.engine_mut().driver_demand = self.driver_demand.unwrap_or(0);
        runtime_state.state.engine_mut().actual_engine = self.actual_engine.unwrap_or(0);
        runtime_state.state.engine_mut().rpm = self.rpm.unwrap_or(0);
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

#[derive(Default)]
pub struct EngineManagementSystem;

impl EngineManagementSystem {
    pub fn serialize(&self, engine_message: &mut EngineMessage) -> Vec<Frame> {
        engine_message.to_frame()
    }
}

impl super::Parsable<EngineMessage> for EngineManagementSystem {
    fn parse(&mut self, frame: &Frame) -> Option<EngineMessage> {
        if frame.len() != 8 {
            return None;
        }
        if frame.id().pgn() != PGN::ElectronicEngineController2 {
            return None;
        }

        Some(EngineMessage::from_frame(frame))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_on() {
        let frame =
            FrameBuilder::new(IdBuilder::from_pgn(PGN::ElectronicEngineController2).build())
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
            FrameBuilder::new(IdBuilder::from_pgn(PGN::ElectronicEngineController2).build())
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
