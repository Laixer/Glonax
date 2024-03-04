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
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        self.ems.try_accept(router, runtime_state).await;
    }

    // FUTURE: Optimize
    async fn tick(
        &self,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        match runtime_state.read().await.governor_mode() {
            crate::core::EngineState::NoRequest => {
                if let Err(e) = router.inner().send(&self.ems.speed_control(0)).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            crate::core::EngineState::Stopping => {
                if let Err(e) = router.inner().send(&self.ems.brake_control()).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            crate::core::EngineState::Starting(rpm) | crate::core::EngineState::Request(rpm) => {
                if let Err(e) = router.inner().send(&self.ems.speed_control(rpm)).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
        }
    }
}
