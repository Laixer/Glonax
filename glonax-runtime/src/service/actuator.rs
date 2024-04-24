use crate::{
    core::Motion,
    runtime::{Service, ServiceContext, SharedOperandState},
};

pub struct ActuatorSimulator {}

impl<C> Service<C> for ActuatorSimulator {
    fn new(_: C) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("actuator simulator")
    }

    async fn on_event(&mut self, runtime_state: SharedOperandState, motion: &crate::core::Motion) {
        match motion {
            Motion::StopAll => {
                runtime_state.write().await.state.ecu_state.lock();
            }
            Motion::ResumeAll => {
                runtime_state.write().await.state.ecu_state.unlock();
            }
            Motion::ResetAll => {
                runtime_state.write().await.state.ecu_state.lock();
                runtime_state.write().await.state.ecu_state.unlock();
            }
            Motion::StraightDrive(_value) => {
                // TODO: Implement, maybe ask ecu_state for straight drive
            }
            Motion::Change(changes) => {
                if runtime_state.read().await.state.ecu_state.is_locked() {
                    return;
                }

                for changeset in changes {
                    runtime_state.write().await.state.ecu_state.speed[changeset.actuator as usize]
                        .store(changeset.value, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }
}