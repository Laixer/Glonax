use glonax::{
    core::Motion,
    runtime::{MotionSender, SharedOperandState},
};

use crate::config::ProxyConfig;

pub type MotionReceiver = tokio::sync::mpsc::Receiver<Motion>;

pub(super) async fn service_net_encoder(
    config: ProxyConfig,
    _instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    _motion_sender: MotionSender,
) {
    use glonax::device::KueblerEncoder;
    use glonax::net::{J1939Network, Router};

    log::debug!("Starting encoder service");

    match J1939Network::new(&config.interface[0], glonax::consts::DEFAULT_J1939_ADDRESS) {
        Ok(network) => {
            let mut router = Router::new(network);

            let mut encoder_list = vec![
                KueblerEncoder::new(0x6A),
                KueblerEncoder::new(0x6B),
                KueblerEncoder::new(0x6C),
                KueblerEncoder::new(0x6D),
            ];

            loop {
                if let Err(e) = router.listen().await {
                    log::error!("Failed to receive from router: {}", e);
                }

                for encoder in &mut encoder_list {
                    if let Some(message) = router.try_accept(encoder) {
                        let mut runtime_state = runtime_state.write().await;

                        runtime_state
                            .state
                            .encoders
                            .insert(message.node, message.position as f32);

                        // TODO: Set the encoder state in the runtime state
                        if let Some(state) = message.state {
                            log::debug!("0x{:X?} Encoder state: {:?}", message.node, state);
                        }
                    }
                }
            }
        }
        Err(e) => log::error!("Failed to open network: {}", e),
    }
}

pub(super) async fn service_net_ems(
    config: ProxyConfig,
    _instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    _motion_sender: MotionSender,
) {
    use glonax::net::{EngineManagementSystem, J1939Network, Router};

    log::debug!("Starting EMS service");

    match J1939Network::new(&config.interface[1], glonax::consts::DEFAULT_J1939_ADDRESS) {
        Ok(network) => {
            let mut router = Router::new(network);

            let mut engine_management_service = EngineManagementSystem;

            loop {
                if let Err(e) = router.listen().await {
                    log::error!("Failed to receive from router: {}", e);
                }

                if let Some(message) = router.try_accept(&mut engine_management_service) {
                    let mut runtime_state = runtime_state.write().await;

                    runtime_state.state.engine.driver_demand = message.driver_demand.unwrap_or(0);
                    runtime_state.state.engine.actual_engine = message.actual_engine.unwrap_or(0);
                    runtime_state.state.engine.rpm = message.rpm.unwrap_or(0);
                }
            }
        }
        Err(e) => {
            log::error!("Failed to open network: {}", e);
            log::warn!("Disabling EMS service");

            runtime_state.write().await.state.engine.status =
                glonax::core::EngineStatus::NetworkDown;
        }
    }
}

pub(super) async fn service_gnss(
    config: ProxyConfig,
    _instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    _motion_sender: MotionSender,
) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    log::debug!("Starting GNSS service");

    match glonax_serial::Uart::open(
        std::path::Path::new(config.nmea_device.as_ref().unwrap()),
        glonax_serial::BaudRate::from_speed(config.nmea_baud_rate),
    ) {
        Ok(serial) => {
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
        }
        Err(e) => {
            log::error!("Failed to open GNSS device: {}", e);
            log::warn!("Disabling GNSS service");

            runtime_state.write().await.state.gnss.status =
                glonax::core::GnssStatus::DeviceNotFound;
        }
    }
}

pub(super) async fn sink_net_actuator_sim(
    _config: ProxyConfig,
    _instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    mut motion_rx: MotionReceiver,
) {
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
}

pub(super) async fn sink_net_actuator(
    config: ProxyConfig,
    _instance: glonax::core::Instance,
    _runtime_state: SharedOperandState,
    mut motion_rx: MotionReceiver,
) {
    use glonax::net::{HydraulicControlUnit, J1939Network};

    log::debug!("Starting motion listener");

    match J1939Network::new(&config.interface[0], glonax::consts::DEFAULT_J1939_ADDRESS) {
        Ok(network) => {
            let service = HydraulicControlUnit::new(0x4A);

            while let Some(motion) = motion_rx.recv().await {
                match motion {
                    Motion::StopAll => {
                        if let Err(e) = network.send_vectored(&service.lock()).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                    Motion::ResumeAll => {
                        if let Err(e) = network.send_vectored(&service.unlock()).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                    Motion::ResetAll => {
                        if let Err(e) = network.send_vectored(&service.lock()).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                        if let Err(e) = network.send_vectored(&service.unlock()).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                    Motion::StraightDrive(value) => {
                        let frames = &service.drive_straight(value);
                        if let Err(e) = network.send_vectored(frames).await {
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

                        if let Err(e) = network.send_vectored(frames).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                }
            }

            log::debug!("Motion listener shutdown");
        }
        Err(e) => log::error!("Failed to open network: {}", e),
    }
}
