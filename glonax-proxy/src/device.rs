use std::time::Duration;

use glonax::core::Motion;
use tokio::time::sleep;

use crate::config::ProxyConfig;

pub type MotionReceiver = tokio::sync::mpsc::Receiver<Motion>;
pub type SharedMachineState = std::sync::Arc<tokio::sync::RwLock<glonax::MachineState>>;

pub(super) async fn service_host(
    local_config: ProxyConfig,
    local_machine_state: SharedMachineState,
) {
    log::debug!("Starting host service");

    let mut service = glonax::net::HostService::new();

    loop {
        service.refresh();
        service.fill(local_machine_state.clone()).await;

        sleep(Duration::from_millis(local_config.host_interval)).await;
    }
}

pub(super) async fn service_net_encoder(
    local_config: ProxyConfig,
    local_machine_state: SharedMachineState,
) {
    use glonax::net::{EncoderService, J1939Network, Router};

    log::debug!("Starting encoder service");

    match J1939Network::new(
        &local_config.interface,
        glonax::consts::DEFAULT_J1939_ADDRESS,
    ) {
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
                        message.fill(local_machine_state.clone()).await;
                    }
                }
            }
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}

pub(super) async fn service_net_ems(
    local_config: ProxyConfig,
    local_machine_state: SharedMachineState,
) {
    if local_config.interface2.is_none() {
        return;
    }

    use glonax::net::{EngineManagementSystem, J1939Network, Router};

    log::debug!("Starting EMS service");

    match J1939Network::new(
        &local_config.interface2.unwrap(),
        glonax::consts::DEFAULT_J1939_ADDRESS,
    ) {
        Ok(network) => {
            let mut router = Router::new(network);

            let mut engine_management_service = EngineManagementSystem::new();

            loop {
                if let Err(e) = router.listen().await {
                    log::error!("Failed to receive from router: {}", e);
                }

                if let Some(message) = router.try_accept(&mut engine_management_service) {
                    message.fill(local_machine_state.clone()).await;
                }
            }
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}

pub(super) async fn service_gnss(
    local_config: ProxyConfig,
    local_machine_state: SharedMachineState,
) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    if local_config.gnss_device.is_none() {
        return;
    }

    log::debug!("Starting GNSS service");

    match glonax_serial::Uart::open(
        &std::path::Path::new(local_config.gnss_device.as_ref().unwrap()),
        glonax_serial::BaudRate::from_speed(local_config.gnss_baud_rate),
    ) {
        Ok(serial) => {
            let reader = BufReader::new(serial);
            let mut lines = reader.lines();

            let service = glonax::net::NMEAService::new();

            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(message) = service.decode(line) {
                    message.fill(local_machine_state.clone()).await;
                }
            }
        }
        Err(e) => {
            log::error!("Failed to open serial: {}", e);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}

pub(super) async fn sink_net_actuator(local_config: ProxyConfig, mut motion_rx: MotionReceiver) {
    use glonax::net::{ActuatorService, J1939Network};

    log::debug!("Starting motion listener");

    match J1939Network::new(
        &local_config.interface,
        glonax::consts::DEFAULT_J1939_ADDRESS,
    ) {
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
