use std::time::Duration;

use glonax::core::Motion;
use tokio::time::sleep;

use crate::{config::ProxyConfig, state::SharedExcavatorState};

pub type MotionReceiver = tokio::sync::mpsc::Receiver<Motion>;

pub(super) async fn service_host(config: ProxyConfig, runtime_state: SharedExcavatorState) {
    log::debug!("Starting host service");

    let mut service = glonax::net::HostService::default();

    loop {
        service.refresh();
        service.fill(runtime_state.clone()).await;

        sleep(Duration::from_millis(config.host_interval)).await;
    }
}

pub(super) async fn service_net_encoder_sim(
    _config: ProxyConfig,
    runtime_state: SharedExcavatorState,
) {
    use glonax::net::EncoderMessage;

    use std::sync::atomic::Ordering;

    log::debug!("Starting encoder service");

    let encoder_frame = glonax::net::Encoder::new(2_500, (0, 6_280), true, false);
    let encoder_boom = glonax::net::Encoder::new(5_000, (0, 1_832), false, false);
    let encoder_arm = glonax::net::Encoder::new(5_000, (685, 2_760), false, true);
    let encoder_attachment = glonax::net::Encoder::new(5_000, (0, 3_100), false, false);

    let mut control_devices = [
        (0x6A, glonax::core::Actuator::Slew, encoder_frame),
        (0x6B, glonax::core::Actuator::Boom, encoder_boom),
        (0x6C, glonax::core::Actuator::Arm, encoder_arm),
        (0x6D, glonax::core::Actuator::Attachment, encoder_attachment),
    ];

    // let mut encoder_list = vec![
    //     EncoderService::new(0x6A),
    //     EncoderService::new(0x6B),
    //     EncoderService::new(0x6C),
    //     EncoderService::new(0x6D),
    // ];

    loop {
        for (id, actuator, encoder) in control_devices.iter_mut() {
            sleep(Duration::from_millis(5)).await;

            // 1st derivative of position
            let velocity = runtime_state.read().await.state.ecu_state.speed[*actuator as usize]
                .load(Ordering::SeqCst);
            let position = runtime_state.read().await.state.ecu_state.position[*actuator as usize]
                .load(Ordering::SeqCst);

            let position = encoder.position(position, velocity);

            EncoderMessage::from_position(*id, position)
                .fill(runtime_state.clone())
                .await;

            runtime_state.write().await.state.ecu_state.position[*actuator as usize]
                .store(position, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

pub(super) async fn service_net_encoder(config: ProxyConfig, runtime_state: SharedExcavatorState) {
    use glonax::net::{EncoderService, J1939Network, Router};

    log::debug!("Starting encoder service");

    match J1939Network::new(&config.interface, glonax::consts::DEFAULT_J1939_ADDRESS) {
        Ok(network) => {
            let mut router = Router::new(network);

            let mut encoder_list = vec![
                EncoderService::new(0x6A),
                EncoderService::new(0x6B),
                EncoderService::new(0x6C),
                EncoderService::new(0x6D),
            ];

            loop {
                if let Err(e) = router.listen().await {
                    log::error!("Failed to receive from router: {}", e);
                }

                for encoder in &mut encoder_list {
                    if let Some(message) = router.try_accept(encoder) {
                        message.fill(runtime_state.clone()).await;
                    }
                }
            }
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}

pub(super) async fn service_net_ems_sim(_config: ProxyConfig, runtime_state: SharedExcavatorState) {
    use glonax::net::EngineMessage;

    log::debug!("Starting EMS service");

    use rand::Rng;
    let mut rng = rand::rngs::OsRng;

    loop {
        sleep(Duration::from_millis(10)).await;

        EngineMessage {
            driver_demand: Some(rng.gen_range(18..=20)),
            actual_engine: Some(rng.gen_range(19..=21)),
            rpm: Some(rng.gen_range(1180..=1200)),
            ..Default::default()
        }
        .fill(runtime_state.clone())
        .await;
    }
}

pub(super) async fn service_net_ems(config: ProxyConfig, runtime_state: SharedExcavatorState) {
    use glonax::net::{EngineManagementSystem, J1939Network, Router};

    if config.interface2.is_none() {
        return;
    }

    log::debug!("Starting EMS service");

    match J1939Network::new(
        &config.interface2.unwrap(),
        glonax::consts::DEFAULT_J1939_ADDRESS,
    ) {
        Ok(network) => {
            let mut router = Router::new(network);

            let mut engine_management_service = EngineManagementSystem;

            loop {
                if let Err(e) = router.listen().await {
                    log::error!("Failed to receive from router: {}", e);
                }

                if let Some(message) = router.try_accept(&mut engine_management_service) {
                    message.fill(runtime_state.clone()).await;
                }
            }
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}

pub(super) async fn service_gnss(config: ProxyConfig, runtime_state: SharedExcavatorState) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    if config.gnss_device.is_none() {
        return;
    }

    log::debug!("Starting GNSS service");

    match glonax_serial::Uart::open(
        std::path::Path::new(config.gnss_device.as_ref().unwrap()),
        glonax_serial::BaudRate::from_speed(config.gnss_baud_rate),
    ) {
        Ok(serial) => {
            let reader = BufReader::new(serial);
            let mut lines = reader.lines();

            let service = glonax::net::NMEAService;

            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(message) = service.decode(line) {
                    message.fill(runtime_state.clone()).await;
                }
            }
        }
        Err(e) => {
            log::error!("Failed to open serial: {}", e);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}

pub(super) async fn sink_net_actuator_sim(
    _config: ProxyConfig,
    runtime_state: SharedExcavatorState,
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
                // TODO: Implement
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
    _runtime_state: SharedExcavatorState,
    mut motion_rx: MotionReceiver,
) {
    use glonax::net::{ActuatorService, J1939Network};

    log::debug!("Starting motion listener");

    match J1939Network::new(&config.interface, glonax::consts::DEFAULT_J1939_ADDRESS) {
        Ok(network) => {
            let service = ActuatorService::new(0x4A);

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
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}
