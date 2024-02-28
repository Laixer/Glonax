use j1939::{
    decode::{self},
    spn, Frame, FrameBuilder, IdBuilder, PDU_MAX_LENGTH, PGN,
};

use crate::net::Parsable;

use super::vecraft::VecraftConfigMessage;

pub enum EngineMessage {
    TorqueSpeedControl(spn::TorqueSpeedControlMessage),
    EngineController(spn::EngineControllerMessage),
}

#[derive(Default)]
pub struct EngineManagementSystem {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
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

        let mut message = spn::TorqueSpeedControlMessage {
            override_control_mode: Some(decode::OverrideControlMode::OverrideDisabled),
            speed_control_condition: None,
            control_mode_priority: None,
            speed: None,
            torque: None,
        };

        if !idle {
            message.override_control_mode = Some(decode::OverrideControlMode::SpeedControl);
            message.speed = Some(rpm);
        }

        let frame = frame_builder.copy_from_slice(&message.to_pdu()).build();
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
        let message = spn::TorqueSpeedControlMessage {
            override_control_mode: Some(decode::OverrideControlMode::SpeedTorqueLimitControl),
            speed_control_condition: None,
            control_mode_priority: None,
            speed: Some(rpm),
            torque: None,
        };

        let frame = frame_builder.copy_from_slice(&message.to_pdu()).build();
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

impl Parsable<EngineMessage> for EngineManagementSystem {
    fn parse(&mut self, frame: &Frame) -> Option<EngineMessage> {
        if frame.id().pgn() == PGN::TorqueSpeedControl1 {
            Some(EngineMessage::TorqueSpeedControl(
                spn::TorqueSpeedControlMessage::from_pdu(frame.pdu()),
            ))
        } else if frame.id().pgn() == PGN::ElectronicEngineController1 {
            if frame.id().sa() != self.destination_address {
                return None;
            }

            Some(EngineMessage::EngineController(
                spn::EngineControllerMessage::from_pdu(frame.pdu()),
            ))
        } else {
            None
        }
    }
}

impl super::J1939Unit for EngineManagementSystem {
    async fn try_accept(
        &mut self,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        if let Some(message) = router.try_accept(self) {
            if let Ok(mut runtime_state) = runtime_state.try_write() {
                match message {
                    EngineMessage::TorqueSpeedControl(_control) => {
                        //
                    }
                    EngineMessage::EngineController(controller) => {
                        runtime_state.state.engine.driver_demand =
                            controller.driver_demand.unwrap_or(0);
                        runtime_state.state.engine.actual_engine =
                            controller.actual_engine.unwrap_or(0);
                        runtime_state.state.engine.rpm = controller.rpm.unwrap_or(0);
                    }
                }
            }
        }
    }

    // FUTURE: Optimize
    async fn tick(
        &self,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        let engine = runtime_state.read().await.state.engine;
        let engine_request = runtime_state.read().await.state.engine_request;

        match engine.mode() {
            crate::core::EngineMode::NoRequest => {
                if engine_request == 0 {
                    if let Err(e) = router
                        .inner()
                        .send_vectored(&self.speed_request(engine_request, true))
                        .await
                    {
                        log::error!("Failed to speed request: {}", e);
                    }
                } else if let Err(e) = router
                    .inner()
                    .send_vectored(&self.start(engine_request))
                    .await
                {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            crate::core::EngineMode::Start => {
                if engine_request == 0 {
                    if let Err(e) = router
                        .inner()
                        .send_vectored(&self.speed_request(engine_request, true))
                        .await
                    {
                        log::error!("Failed to speed request: {}", e);
                    }
                } else if let Err(e) = router
                    .inner()
                    .send_vectored(&self.start(engine_request))
                    .await
                {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            crate::core::EngineMode::Idle | crate::core::EngineMode::Running => {
                if engine_request == 0 {
                    if let Err(e) = router.inner().send_vectored(&self.shutdown()).await {
                        log::error!("Failed to speed request: {}", e);
                    }
                } else if let Err(e) = router
                    .inner()
                    .send_vectored(&self.speed_request(engine_request, false))
                    .await
                {
                    log::error!("Failed to speed request: {}", e);
                }
            }
        }
    }
}
