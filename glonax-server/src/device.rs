use glonax::{core::Motion, runtime::SharedOperandState};

pub type MotionReceiver = tokio::sync::mpsc::Receiver<Motion>;

pub(super) async fn sink_net_actuator_sim(
    _config: crate::config::Config,
    _instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    mut motion_rx: MotionReceiver,
) -> std::io::Result<()> {
    log::debug!("Starting motion listener");

    while let Some(motion) = motion_rx.recv().await {
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
                    continue;
                }

                for changeset in &changes {
                    runtime_state.write().await.state.ecu_state.speed[changeset.actuator as usize]
                        .store(changeset.value, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }

    Ok(())
}
