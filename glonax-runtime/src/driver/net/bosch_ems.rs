use j1939::Frame;

use crate::{driver::EngineMessage, net::Parsable};

use super::engine::EngineManagementSystem;

#[derive(Default)]
pub struct BoschEngineManagementSystem {
    /// Engine management system.
    ems: EngineManagementSystem,
}

impl BoschEngineManagementSystem {
    /// Construct a new engine management system.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            ems: EngineManagementSystem::new(da, sa),
        }
    }
}

impl super::engine::Engine for BoschEngineManagementSystem {
    fn request(&self, speed: u16) -> Frame {
        self.ems.request(speed)
    }

    fn start(&self, speed: u16) -> Frame {
        self.ems.speed_control(speed)
    }

    fn stop(&self, _speed: u16) -> Frame {
        self.ems.brake_control()
    }
}

impl Parsable<EngineMessage> for BoschEngineManagementSystem {
    fn parse(&mut self, frame: &Frame) -> Option<EngineMessage> {
        self.ems.parse(frame)
    }
}

impl super::J1939Unit for BoschEngineManagementSystem {
    fn name(&self) -> &str {
        "Bosch ECM"
    }

    fn destination(&self) -> u8 {
        self.ems.destination()
    }

    async fn setup(
        &self,
        ctx: &mut super::NetDriverContext,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        self.ems.setup(ctx, router, runtime_state).await
    }

    async fn teardown(
        &self,
        ctx: &mut super::NetDriverContext,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        self.ems.teardown(ctx, router, runtime_state).await
    }

    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        self.ems.try_accept(ctx, router, runtime_state).await
    }

    async fn tick(
        &self,
        ctx: &mut super::NetDriverContext,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        self.ems.tick(ctx, router, runtime_state).await
    }
}
