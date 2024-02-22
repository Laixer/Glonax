use glonax::{
    core::Motion,
    runtime::{MotionSender, SharedOperandState},
};

pub type MotionReceiver = tokio::sync::mpsc::Receiver<Motion>;

pub(super) async fn _service_gnss(
    config: crate::config::Config,
    _instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    _motion_sender: MotionSender,
) -> std::io::Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    log::debug!("Starting GNSS service");

    let nmea_config = config.nmea.as_ref().unwrap();

    let serial = glonax_serial::Uart::open(
        std::path::Path::new(&nmea_config.device),
        glonax_serial::BaudRate::from_speed(nmea_config.baud_rate),
    )
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let reader = BufReader::new(serial);
    let mut lines = reader.lines();

    let driver = glonax::driver::Nmea;

    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(message) = driver.decode(line) {
            let mut runtime_state = runtime_state.write().await;

            if let Some((lat, long)) = message.coordinates {
                runtime_state.state.gnss.location = (lat, long)
            }
            if let Some(altitude) = message.altitude {
                runtime_state.state.gnss.altitude = altitude;
            }
            if let Some(speed) = message.speed {
                const KNOT_TO_METER_PER_SECOND: f32 = 0.5144;

                runtime_state.state.gnss.speed = speed * KNOT_TO_METER_PER_SECOND;
            }
            if let Some(heading) = message.heading {
                runtime_state.state.gnss.heading = heading;
            }
            if let Some(satellites) = message.satellites {
                runtime_state.state.gnss.satellites = satellites;
            }
        }
    }

    Ok(())
}

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
