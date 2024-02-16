use j1939::{
    decode::{self},
    spn, Frame, FrameBuilder, IdBuilder, PDU_MAX_LENGTH, PGN,
};

use crate::net::Parsable;

use super::vecraft::VecraftConfigMessage;

// impl std::fmt::Display for EngineControllerMessage {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut s = String::new();

//         if let Some(engine_torque_mode) = &self.engine_torque_mode {
//             s.push_str(&format!("Torque mode: {:?}; ", engine_torque_mode));
//         }

//         if let Some(driver_demand) = self.driver_demand {
//             s.push_str(&format!("Driver Demand: {}%; ", driver_demand));
//         }

//         if let Some(actual_engine) = self.actual_engine {
//             s.push_str(&format!("Actual Engine: {}%; ", actual_engine));
//         }

//         if let Some(rpm) = self.rpm {
//             s.push_str(&format!("RPM: {}; ", rpm));
//         }

//         if let Some(starter_mode) = &self.starter_mode {
//             s.push_str(&format!("Starter mode: {:?}; ", starter_mode));
//         }

//         write!(f, "{}", s)
//     }
// }

#[derive(Default)]
pub struct EngineManagementSystem {
    /// Destination address.
    pub destination_address: u8,
    /// Source address.
    pub source_address: u8,
}

impl EngineManagementSystem {
    /// Construct a new engine management system.
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

        frame_builder.as_mut()[3] = 0b0001_0000;

        vec![frame_builder.set_len(PDU_MAX_LENGTH).build()]
    }
}

impl Parsable<spn::EngineControllerMessage> for EngineManagementSystem {
    fn parse(&mut self, frame: &Frame) -> Option<spn::EngineControllerMessage> {
        if frame.id().pgn() != PGN::ElectronicEngineController1 {
            return None;
        }
        if frame.id().sa() != self.destination_address {
            return None;
        }

        Some(spn::EngineControllerMessage::from_pdu(frame.pdu()))
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
