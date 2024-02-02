use glonax::{
    core::Motion,
    runtime::{MotionSender, SharedOperandState},
};

use crate::config::ProxyConfig;

pub type MotionReceiver = tokio::sync::mpsc::Receiver<Motion>;

pub(super) async fn service_gnss(
    config: ProxyConfig,
    _instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    _motion_sender: MotionSender,
) -> std::io::Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    log::debug!("Starting GNSS service");

    let serial = glonax_serial::Uart::open(
        std::path::Path::new(config.nmea_device.as_ref().unwrap()),
        glonax_serial::BaudRate::from_speed(config.nmea_baud_rate),
    )
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let reader = BufReader::new(serial);
    let mut lines = reader.lines();

    let driver = glonax::device::Nmea;

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
    _config: ProxyConfig,
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

pub(super) async fn sink_net_actuator(
    config: ProxyConfig,
    _instance: glonax::core::Instance,
    _runtime_state: SharedOperandState,
    mut motion_rx: MotionReceiver,
) -> std::io::Result<()> {
    use glonax::device::HydraulicControlUnit;
    use glonax::net::J1939Network;

    log::debug!("Starting motion listener");

    let net = J1939Network::new(&config.interface[0], glonax::consts::DEFAULT_J1939_ADDRESS)?;

    let service = HydraulicControlUnit::new(0x4A);

    while let Some(motion) = motion_rx.recv().await {
        match motion {
            Motion::StopAll => {
                if let Err(e) = net.send_vectored(&service.lock()).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            Motion::ResumeAll => {
                if let Err(e) = net.send_vectored(&service.unlock()).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            Motion::ResetAll => {
                if let Err(e) = net.send_vectored(&service.motion_reset()).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            Motion::StraightDrive(value) => {
                let frames = &service.drive_straight(value);
                if let Err(e) = net.send_vectored(frames).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            Motion::Change(changes) => {
                let frames = &service.actuator_command(
                    changes
                        .iter()
                        .map(|changeset| (changeset.actuator as u8, changeset.value))
                        .collect(),
                );

                if let Err(e) = net.send_vectored(frames).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
        }
    }

    log::debug!("Motion listener shutdown");

    Ok(())
}
