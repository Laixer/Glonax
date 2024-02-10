use j1939::{
    decode::{self, EngineStarterMode, EngineTorqueMode},
    spn, Frame, FrameBuilder, IdBuilder, PDU_MAX_LENGTH, PGN,
};

use crate::net::Parsable;

use super::vecraft::VecraftConfigMessage;

// TODO: Move to j1939 crate
#[derive(Default)]
pub struct EngineControllerMessage {
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

impl EngineControllerMessage {
    // TODO: Move to j1939 crate
    pub fn from_frame(frame: &Frame) -> Self {
        Self {
            engine_torque_mode: decode::spn899(frame.pdu()[0]),
            driver_demand: spn::byte::dec(frame.pdu()[1]),
            actual_engine: spn::byte::dec(frame.pdu()[2]),
            rpm: spn::rpm::dec(&frame.pdu()[3..5]),
            source_addr: decode::spn1483(frame.pdu()[5]),
            starter_mode: decode::spn1675(frame.pdu()[6]),
        }
    }

    // TODO: Move to j1939 crate
    #[allow(dead_code)]
    fn to_frame(&self) -> Vec<Frame> {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ElectronicEngineController1)
                .da(0xff)
                .build(),
        );

        if let Some(driver_demand) = self.driver_demand {
            frame_builder.as_mut()[1] = spn::byte::enc(driver_demand);
        }
        if let Some(actual_engine) = self.actual_engine {
            frame_builder.as_mut()[2] = spn::byte::enc(actual_engine);
        }
        if let Some(rpm) = self.rpm {
            frame_builder.as_mut()[3..5].copy_from_slice(&spn::rpm::enc(rpm));
        }
        if let Some(source_addr) = self.source_addr {
            frame_builder.as_mut()[5] = source_addr;
        }

        vec![frame_builder.set_len(PDU_MAX_LENGTH).build()]
    }
}

impl std::fmt::Display for EngineControllerMessage {
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
pub struct EngineManagementSystem {
    /// Destination address.
    pub destination_address: u8,
    /// Source address.
    pub source_address: u8,
}

impl EngineManagementSystem {
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
        }
    }

    /// Set or unset identification mode.
    pub fn set_ident(&self, on: bool) -> Vec<Frame> {
        VecraftConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: Some(on),
            reboot: false,
        }
        .to_frame()
    }

    /// System reboot / reset
    pub fn reboot(&self) -> Vec<Frame> {
        VecraftConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: None,
            reboot: true,
        }
        .to_frame()
    }

    /// Request speed control
    pub fn speed_request(&self, rpm: u16, idle: bool) -> Vec<Frame> {
        let frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TorqueSpeedControl1)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        );

        let mut msg = j1939::spn::TorqueSpeedControlMessage {
            override_control_mode: Some(decode::OverrideControlMode::OverrideDisabled),
            speed_control_condition: None,
            control_mode_priority: None,
            speed: None,
            torque: None,
        };

        if !idle {
            msg.override_control_mode = Some(decode::OverrideControlMode::SpeedControl);
            msg.speed = Some(rpm);
        }

        let frame = frame_builder.copy_from_slice(&msg.to_pdu()).build();
        vec![frame]
    }

    pub fn start(&self, rpm: u16) -> Vec<Frame> {
        let frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TorqueSpeedControl1)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        );

        // TODO: This is not correct. 0x3 is not used for starting the engine.
        let msg = j1939::spn::TorqueSpeedControlMessage {
            override_control_mode: Some(decode::OverrideControlMode::SpeedTorqueLimitControl),
            speed_control_condition: None,
            control_mode_priority: None,
            speed: Some(rpm),
            torque: None,
        };

        let frame = frame_builder.copy_from_slice(&msg.to_pdu()).build();
        vec![frame]
    }

    pub fn shutdown(&self) -> Vec<Frame> {
        // TODO: Make this a J1939 message
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ElectronicBrakeController1)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        );

        frame_builder.as_mut()[3] = 0b00010000;

        vec![frame_builder.set_len(PDU_MAX_LENGTH).build()]
    }
}

impl Parsable<EngineControllerMessage> for EngineManagementSystem {
    fn parse(&mut self, frame: &Frame) -> Option<EngineControllerMessage> {
        if frame.len() != PDU_MAX_LENGTH {
            return None;
        }
        if frame.id().pgn() != PGN::ElectronicEngineController1 {
            return None;
        }

        Some(EngineControllerMessage::from_frame(frame))
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

        let engine_message = EngineControllerMessage::from_frame(&frame);
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

        let engine_message = EngineControllerMessage::from_frame(&frame);
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
