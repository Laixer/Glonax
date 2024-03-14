use j1939::{Frame, FrameBuilder, IdBuilder, PGN};

use crate::{driver::EngineMessage, net::Parsable};

use super::{engine::EngineManagementSystem, vecraft::VecraftConfigMessage};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum VolvoEngineState {
    /// Engine shutdown.
    Shutdown = 0b0000_0111,
    /// Engine starter locked.
    Locked = 0b0100_0111,
    /// Engine running at requested speed.
    Nominal = 0b0100_0011,
    /// Engine starter engaged.
    Starting = 0b1100_0011,
}

#[derive(Default)]
pub struct VolvoD7E {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
    /// Engine management system.
    ems: EngineManagementSystem,
}

impl VolvoD7E {
    /// Construct a new engine management system.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
            ems: EngineManagementSystem::new(da, sa),
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
    pub fn speed_control(&self, state: VolvoEngineState, rpm: u16) -> Frame {
        FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietaryB(65_282))
                .priority(3)
                .sa(self.source_address)
                .build(),
        )
        .copy_from_slice(&[
            0x00,
            state as u8,
            0x1f,
            0x00,
            0x00,
            0x00,
            0x20,
            (rpm as f32 / 10.0) as u8,
        ])
        .build()
    }

    // pub fn start(&self, rpm: u16) -> Frame {
    //     let frame_builder = FrameBuilder::new(
    //         IdBuilder::from_pgn(PGN::TorqueSpeedControl1)
    //             .priority(3)
    //             .da(self.destination_address)
    //             .sa(self.source_address)
    //             .build(),
    //     );

    //     // TODO: This is not correct. 'SpeedTorqueLimitControl' is not used for starting the engine.
    //     // Change to proprietary PGN.
    //     frame_builder
    //         .copy_from_slice(
    //             &spn::TorqueSpeedControl1Message {
    //                 override_control_mode: Some(spn::OverrideControlMode::SpeedTorqueLimitControl),
    //                 speed_control_condition: None,
    //                 control_mode_priority: None,
    //                 speed: Some(rpm),
    //                 torque: None,
    //             }
    //             .to_pdu(),
    //         )
    //         .build()
    // }
}

impl Parsable<EngineMessage> for VolvoD7E {
    fn parse(&mut self, frame: &Frame) -> Option<EngineMessage> {
        self.ems.parse(frame)
    }
}

impl super::J1939Unit for VolvoD7E {
    async fn try_accept(
        &mut self,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        self.ems.try_accept(state, router, runtime_state).await;
    }

    async fn tick(
        &self,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        if state == &super::J1939UnitOperationState::Running {
            let request = runtime_state.read().await.governor_mode();
            match request.state {
                crate::core::EngineState::NoRequest => {
                    if let Err(e) = router
                        .send(&self.speed_control(VolvoEngineState::Nominal, request.speed))
                        .await
                    {
                        log::error!("Failed to speed request: {}", e);
                    }
                }
                crate::core::EngineState::Starting => {
                    if let Err(e) = router
                        .send(&self.speed_control(VolvoEngineState::Starting, request.speed))
                        .await
                    {
                        log::error!("Failed to speed request: {}", e);
                    }
                }
                crate::core::EngineState::Stopping => {
                    if let Err(e) = router
                        .send(&self.speed_control(VolvoEngineState::Shutdown, request.speed))
                        .await
                    {
                        log::error!("Failed to speed request: {}", e);
                    }
                }
                crate::core::EngineState::Request => {
                    if let Err(e) = router
                        .send(&self.speed_control(VolvoEngineState::Nominal, request.speed))
                        .await
                    {
                        log::error!("Failed to speed request: {}", e);
                    }
                }
            }
        }
    }
}
