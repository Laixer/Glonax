use j1939::{
    decode::{self, EngineStarterMode, EngineTorqueMode},
    spn, Frame, FrameBuilder, IdBuilder, PDU_MAX_LENGTH, PGN,
};

use crate::net::Parsable;

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

// TODO: Move to j1939 crate
#[allow(dead_code)]
struct TorqueSpeedControlMessage {
    /// Destination address
    destination_address: u8,
    /// Source address
    source_address: u8,
    /// Override control mode - SPN 695
    override_control_mode: Option<decode::OverrideControlMode>,
    /// Requested speed control conditions - SPN 696
    speed_control_condition: Option<decode::RequestedSpeedControlCondition>,
    /// Override control mode priority - SPN 897
    control_mode_priority: Option<decode::OverrideControlModePriority>,
    /// Requested speed or speed limit - SPN 898
    speed: Option<u16>,
    /// Requested torque or torque limit - SPN 518
    torque: Option<u8>,
}

impl TorqueSpeedControlMessage {
    // TODO: Move to j1939 crate
    fn to_frame(&self) -> Vec<Frame> {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::TorqueSpeedControl1)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        );

        frame_builder.as_mut()[0] = match self.override_control_mode {
            Some(decode::OverrideControlMode::OverrideDisabled) => 0b00,
            Some(decode::OverrideControlMode::SpeedControl) => 0b01,
            Some(decode::OverrideControlMode::TorqueControl) => 0b10,
            Some(decode::OverrideControlMode::SpeedTorqueLimitControl) => 0b11,
            None => 0b00,
        };

        if let Some(speed) = self.speed {
            frame_builder.as_mut()[1..3].copy_from_slice(&spn::rpm::enc(speed));
        }

        if let Some(torque) = self.torque {
            frame_builder.as_mut()[3] = torque;
        }

        vec![frame_builder.set_len(PDU_MAX_LENGTH).build()]
    }
}

struct ConfigMessage {
    /// Destination address
    destination_address: u8,
    /// Source address
    source_address: u8,
    /// Identification mode
    pub ident_on: Option<bool>,
    /// Reset hardware
    pub reset: Option<bool>,
}

impl ConfigMessage {
    #[allow(dead_code)]
    fn from_frame(destination_address: u8, source_address: u8, frame: &Frame) -> Self {
        let mut ident_on = None;
        let mut reset = None;

        if frame.pdu()[2] == 0x0 {
            ident_on = Some(false);
        } else if frame.pdu()[2] == 0x1 {
            ident_on = Some(true);
        }
        if frame.pdu()[3] == 0x0 {
            reset = Some(false);
        } else if frame.pdu()[3] == 0x69 {
            reset = Some(true);
        }

        Self {
            destination_address,
            source_address,
            ident_on,
            reset,
        }
    }

    fn to_frame(&self) -> Vec<Frame> {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage1)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', 0xff, 0xff]);

        if let Some(led_on) = self.ident_on {
            frame_builder.as_mut()[2] = u8::from(led_on);
        }

        if let Some(reset) = self.reset {
            frame_builder.as_mut()[3] = if reset { 0x69 } else { 0x0 };
        }

        vec![frame_builder.build()]
    }
}

impl std::fmt::Display for ConfigMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config {} {}",
            if self.ident_on.unwrap_or(false) {
                "Ident on"
            } else {
                "Ident off"
            },
            if self.reset.unwrap_or(false) {
                "reset"
            } else {
                "no reset"
            }
        )
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
        ConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: Some(on),
            reset: None,
        }
        .to_frame()
    }

    /// System reboot / reset
    pub fn reboot(&self) -> Vec<Frame> {
        ConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: None,
            reset: Some(true),
        }
        .to_frame()
    }

    /// Request speed control
    pub fn speed_request(&self, rpm: u16) -> Vec<Frame> {
        TorqueSpeedControlMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            override_control_mode: Some(decode::OverrideControlMode::SpeedControl),
            speed_control_condition: None,
            control_mode_priority: None,
            speed: Some(rpm),
            torque: None,
        }
        .to_frame()
    }

    pub fn start(&self) -> Vec<Frame> {
        // TODO: This is not correct. 0x3 is not used for starting the engine.
        TorqueSpeedControlMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            override_control_mode: Some(decode::OverrideControlMode::SpeedTorqueLimitControl),
            speed_control_condition: None,
            control_mode_priority: None,
            speed: Some(700),
            torque: None,
        }
        .to_frame()
    }

    pub fn shutdown(&self) -> Vec<Frame> {
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
