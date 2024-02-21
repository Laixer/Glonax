use std::time::Duration;

use glonax::j1939::{protocol, NameBuilder};
use glonax::runtime::SharedOperandState;

use glonax::driver::net::J1939Unit;
use glonax::driver::{
    EngineManagementSystem, HydraulicControlUnit, KueblerEncoder, KueblerInclinometer,
    RequestResponder,
};
use glonax::net::{CANSocket, Router, SockAddrCAN};

/// J1939 name manufacturer code.
const J1939_NAME_MANUFACTURER_CODE: u16 = 0x717;
/// J1939 name function instance.
const J1939_NAME_FUNCTION_INSTANCE: u8 = 6;
/// J1939 name ECU instance.
const J1939_NAME_ECU_INSTANCE: u8 = 0;
/// J1939 name function.
const J1939_NAME_FUNCTION: u8 = 0x1C;
/// J1939 name vehicle system.
const J1939_NAME_VEHICLE_SYSTEM: u8 = 2;

// TODO: Move into runtime
pub(super) async fn rx_network_0(
    interface: String,
    runtime_state: SharedOperandState,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;
    let mut router = Router::new(socket);

    let mut enc0 = KueblerEncoder::new(
        crate::consts::J1939_ADDRESS_ENCODER0,
        crate::consts::J1939_ADDRESS_VMS,
    );
    let mut enc1 = KueblerEncoder::new(
        crate::consts::J1939_ADDRESS_ENCODER1,
        crate::consts::J1939_ADDRESS_VMS,
    );
    let mut enc2 = KueblerEncoder::new(
        crate::consts::J1939_ADDRESS_ENCODER2,
        crate::consts::J1939_ADDRESS_VMS,
    );
    let mut enc3 = KueblerEncoder::new(
        crate::consts::J1939_ADDRESS_ENCODER3,
        crate::consts::J1939_ADDRESS_VMS,
    );
    let mut imu0 = KueblerInclinometer::new(
        crate::consts::J1939_ADDRESS_IMU0,
        crate::consts::J1939_ADDRESS_VMS,
    );
    let mut rrp0 = RequestResponder::new(crate::consts::J1939_ADDRESS_VMS);

    while shutdown.is_empty() {
        if let Err(e) = router.listen().await {
            log::error!("Failed to receive from router: {}", e);
        }

        enc0.try_accept(&mut router, runtime_state.clone()).await;
        enc1.try_accept(&mut router, runtime_state.clone()).await;
        enc2.try_accept(&mut router, runtime_state.clone()).await;
        enc3.try_accept(&mut router, runtime_state.clone()).await;
        imu0.try_accept(&mut router, runtime_state.clone()).await;
        rrp0.try_accept(&mut router, runtime_state.clone()).await;
    }

    Ok(())
}

pub(super) async fn rx_network_1(
    interface: String,
    runtime_state: SharedOperandState,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;
    let mut router = Router::new(socket);

    let mut ems0 = EngineManagementSystem::new(
        crate::consts::J1939_ADDRESS_ENGINE0,
        crate::consts::J1939_ADDRESS_VMS,
    );
    let mut hcu0 = HydraulicControlUnit::new(
        crate::consts::J1939_ADDRESS_HCU0,
        crate::consts::J1939_ADDRESS_VMS,
    );
    let mut rrp0 = RequestResponder::new(crate::consts::J1939_ADDRESS_VMS);

    while shutdown.is_empty() {
        if let Err(e) = router.listen().await {
            log::error!("Failed to receive from router: {}", e);
        }

        ems0.try_accept(&mut router, runtime_state.clone()).await;
        hcu0.try_accept(&mut router, runtime_state.clone()).await;
        rrp0.try_accept(&mut router, runtime_state.clone()).await;
    }

    Ok(())
}

// TODO: Move into runtime
pub(super) async fn atx_network_1(
    interface: String,
    runtime_state: SharedOperandState,
    mut motion_rx: crate::device::MotionReceiver,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;

    let hcu0 = HydraulicControlUnit::new(
        crate::consts::J1939_ADDRESS_HCU0,
        crate::consts::J1939_ADDRESS_VMS,
    );

    while let Some(motion) = motion_rx.recv().await {
        runtime_state.write().await.state.motion = motion.clone();
        match &motion {
            glonax::core::Motion::StopAll => {
                if let Err(e) = socket.send_vectored(&hcu0.lock()).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            glonax::core::Motion::ResumeAll => {
                if let Err(e) = socket.send_vectored(&hcu0.unlock()).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            glonax::core::Motion::ResetAll => {
                if let Err(e) = socket.send_vectored(&hcu0.motion_reset()).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            glonax::core::Motion::StraightDrive(value) => {
                let frames = &hcu0.drive_straight(*value);
                if let Err(e) = socket.send_vectored(frames).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            glonax::core::Motion::Change(changes) => {
                let frames = &hcu0.actuator_command(
                    changes
                        .iter()
                        .map(|changeset| (changeset.actuator as u8, changeset.value))
                        .collect(),
                );

                if let Err(e) = socket.send_vectored(frames).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
        }
    }

    Ok(())
}

// TODO: Move into runtime
pub(super) async fn tx_network_0(
    interface: String,
    _runtime_state: SharedOperandState,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;

    let name = NameBuilder::default()
        .identity_number(0x1)
        .manufacturer_code(J1939_NAME_MANUFACTURER_CODE)
        .function_instance(J1939_NAME_FUNCTION_INSTANCE)
        .ecu_instance(J1939_NAME_ECU_INSTANCE)
        .function(J1939_NAME_FUNCTION)
        .vehicle_system(J1939_NAME_VEHICLE_SYSTEM)
        .build();

    let mut interval = tokio::time::interval(Duration::from_millis(10));

    socket
        .send(&protocol::address_claimed(
            crate::consts::J1939_ADDRESS_VMS,
            name,
        ))
        .await?;

    while shutdown.is_empty() {
        interval.tick().await;
    }

    Ok(())
}

// TODO: Move into runtime
pub(super) async fn tx_network_1(
    interface: String,
    runtime_state: SharedOperandState,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;

    let name = NameBuilder::default()
        .identity_number(0x1)
        .manufacturer_code(J1939_NAME_MANUFACTURER_CODE)
        .function_instance(J1939_NAME_FUNCTION_INSTANCE)
        .ecu_instance(J1939_NAME_ECU_INSTANCE)
        .function(J1939_NAME_FUNCTION)
        .vehicle_system(J1939_NAME_VEHICLE_SYSTEM)
        .build();

    let ems0 = EngineManagementSystem::new(
        crate::consts::J1939_ADDRESS_ENGINE0,
        crate::consts::J1939_ADDRESS_VMS,
    );
    let hcu0 = HydraulicControlUnit::new(
        crate::consts::J1939_ADDRESS_HCU0,
        crate::consts::J1939_ADDRESS_VMS,
    );

    let mut interval = tokio::time::interval(Duration::from_millis(10));

    socket
        .send(&protocol::address_claimed(
            crate::consts::J1939_ADDRESS_VMS,
            name,
        ))
        .await?;

    while shutdown.is_empty() {
        interval.tick().await;

        let engine = runtime_state.read().await.state.engine;
        let engine_request = runtime_state.read().await.state.engine_request;

        match engine.mode() {
            glonax::core::EngineMode::Shutdown => {
                if engine_request == 0 {
                    if let Err(e) = socket
                        .send_vectored(&ems0.speed_request(engine_request, true))
                        .await
                    {
                        log::error!("Failed to speed request: {}", e);
                    }
                } else if let Err(e) = socket.send_vectored(&ems0.start(engine_request)).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            glonax::core::EngineMode::Startup => {
                if engine_request == 0 {
                    if let Err(e) = socket
                        .send_vectored(&ems0.speed_request(engine_request, true))
                        .await
                    {
                        log::error!("Failed to speed request: {}", e);
                    }
                } else if let Err(e) = socket.send_vectored(&ems0.start(engine_request)).await {
                    log::error!("Failed to speed request: {}", e);
                }
            }
            glonax::core::EngineMode::Idle | glonax::core::EngineMode::Running => {
                if engine_request == 0 {
                    if let Err(e) = socket.send_vectored(&ems0.shutdown()).await {
                        log::error!("Failed to speed request: {}", e);
                    }
                } else if let Err(e) = socket
                    .send_vectored(&ems0.speed_request(engine_request, false))
                    .await
                {
                    log::error!("Failed to speed request: {}", e);
                }
            }
        }

        match &runtime_state.read().await.state.motion {
            glonax::core::Motion::StopAll => {
                if let Err(e) = socket.send_vectored(&hcu0.lock()).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            glonax::core::Motion::ResumeAll => {
                if let Err(e) = socket.send_vectored(&hcu0.unlock()).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            glonax::core::Motion::ResetAll => {
                if let Err(e) = socket.send_vectored(&hcu0.motion_reset()).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            glonax::core::Motion::StraightDrive(value) => {
                let frames = &hcu0.drive_straight(*value);
                if let Err(e) = socket.send_vectored(frames).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
            glonax::core::Motion::Change(changes) => {
                let frames = &hcu0.actuator_command(
                    changes
                        .iter()
                        .map(|changeset| (changeset.actuator as u8, changeset.value))
                        .collect(),
                );

                if let Err(e) = socket.send_vectored(frames).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }
        }
    }

    if let Err(e) = socket.send_vectored(&hcu0.lock()).await {
        log::error!("Failed to send motion: {}", e);
    }

    Ok(())
}
