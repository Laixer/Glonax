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

impl Parsable<EngineMessage> for BoschEngineManagementSystem {
    fn parse(&mut self, frame: &Frame) -> Option<EngineMessage> {
        self.ems.parse(frame)
    }
}

impl super::J1939Unit for BoschEngineManagementSystem {
    async fn try_accept(
        &mut self,
        state: &super::J1939UnitOperationState,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        self.ems.try_accept(state, router, runtime_state).await;
    }

    // FUTURE: Optimize
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
                    if let Err(e) = router.send(&self.ems.speed_control(0)).await {
                        log::error!("Failed to speed request: {}", e);
                    }
                }
                crate::core::EngineState::Stopping => {
                    if let Err(e) = router.send(&self.ems.brake_control()).await {
                        log::error!("Failed to speed request: {}", e);
                    }
                }
                crate::core::EngineState::Starting | crate::core::EngineState::Request => {
                    if let Err(e) = router.send(&self.ems.speed_control(request.speed)).await {
                        log::error!("Failed to speed request: {}", e);
                    }
                }
            }
        }
    }
}
