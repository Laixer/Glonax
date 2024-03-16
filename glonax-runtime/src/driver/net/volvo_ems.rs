use j1939::{Frame, FrameBuilder, IdBuilder, PGN};

use crate::{driver::EngineMessage, net::Parsable};

use super::engine::EngineManagementSystem;

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
    _destination_address: u8,
    /// Source address.
    source_address: u8,
    /// Engine management system.
    ems: EngineManagementSystem,
}

impl VolvoD7E {
    /// Construct a new engine management system.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            _destination_address: da,
            source_address: sa,
            ems: EngineManagementSystem::new(da, sa),
        }
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

impl super::engine::Engine for VolvoD7E {
    fn request(&self, speed: u16) -> Frame {
        self.speed_control(VolvoEngineState::Nominal, speed)
    }

    fn start(&self, speed: u16) -> Frame {
        self.speed_control(VolvoEngineState::Starting, speed)
    }

    fn stop(&self, speed: u16) -> Frame {
        self.speed_control(VolvoEngineState::Shutdown, speed)
    }
}

impl Parsable<EngineMessage> for VolvoD7E {
    fn parse(&mut self, frame: &Frame) -> Option<EngineMessage> {
        self.ems.parse(frame)
    }
}

impl super::J1939Unit for VolvoD7E {
    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        self.ems.try_accept(ctx, state, router, runtime_state).await
    }

    async fn tick(
        &self,
        ctx: &mut super::NetDriverContext,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        use super::engine::Engine;

        if state == &super::J1939UnitOperationState::Running {
            let request = runtime_state.read().await.governor_mode();
            match request.state {
                crate::core::EngineState::NoRequest => {
                    router.send(&self.request(request.speed)).await?;
                }
                crate::core::EngineState::Starting => {
                    router.send(&self.start(request.speed)).await?;
                }
                crate::core::EngineState::Stopping => {
                    router.send(&self.stop(request.speed)).await?;
                }
                crate::core::EngineState::Request => {
                    router.send(&self.request(request.speed)).await?;
                }
            }

            if ctx.rx_last.elapsed().as_millis() > 500 {
                Err(super::J1939UnitError::MessageTimeout)?;
            }
        }

        Ok(())
    }
}
