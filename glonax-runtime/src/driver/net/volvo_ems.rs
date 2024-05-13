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
    const VENDOR: &'static str = "volvo";
    const PRODUCT: &'static str = "d7e";

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        self.ems.try_accept(ctx, network, runtime_state).await
    }

    async fn tick(
        &self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        use super::engine::Engine;

        if let Ok(request) = runtime_state.try_read() {
            let request = request.governor_mode();
            match request.state {
                crate::core::EngineState::NoRequest => {
                    network.send(&self.request(request.rpm)).await?;
                    ctx.tx_mark();
                }
                crate::core::EngineState::Starting => {
                    network.send(&self.start(request.rpm)).await?;
                    ctx.tx_mark();
                }
                crate::core::EngineState::Stopping => {
                    network.send(&self.stop(request.rpm)).await?;
                    ctx.tx_mark();
                }
                crate::core::EngineState::Request => {
                    network.send(&self.request(request.rpm)).await?;
                    ctx.tx_mark();
                }
            }
        }

        Ok(())
    }

    // TODO: Accept trigger with engine message
}
