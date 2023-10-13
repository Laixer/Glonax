use std::time::Duration;

use tokio::time::sleep;

use crate::config::ProxyConfig;

pub(super) async fn service_host(
    local_config: ProxyConfig,
    local_machine_state: crate::component::SharedMachineState,
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
    local_machine_state: crate::component::SharedMachineState,
) {
    use glonax::net::{EncoderService, J1939Network, Router};

    log::debug!("Starting encoder services");

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
    local_machine_state: crate::component::SharedMachineState,
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
    local_machine_state: crate::component::SharedMachineState,
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
