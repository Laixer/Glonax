use glonax::runtime::SharedOperandState;

use glonax::device::net::J1939Unit;
use glonax::device::{EngineManagementSystem, HydraulicControlUnit, KueblerEncoder};
use glonax::net::{CANSocket, Router, SockAddrCAN};

// TODO: Move into runtime
pub(super) async fn rx_network_0(
    interface: String,
    runtime_state: SharedOperandState,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;
    let mut router = Router::new(socket);

    let mut enc0 = KueblerEncoder::new(crate::consts::J1939_ADDRESS_ENCODER0);
    let mut enc1 = KueblerEncoder::new(crate::consts::J1939_ADDRESS_ENCODER1);
    let mut enc2 = KueblerEncoder::new(crate::consts::J1939_ADDRESS_ENCODER2);
    let mut enc3 = KueblerEncoder::new(crate::consts::J1939_ADDRESS_ENCODER3);
    let mut hcu0 = HydraulicControlUnit::new(
        crate::consts::J1939_ADDRESS_HCU0,
        crate::consts::J1939_ADDRESS_VMS,
    );

    loop {
        if let Err(e) = router.listen().await {
            log::error!("Failed to receive from router: {}", e);
        }

        enc0.try_accept(&mut router, runtime_state.clone());
        enc1.try_accept(&mut router, runtime_state.clone());
        enc2.try_accept(&mut router, runtime_state.clone());
        enc3.try_accept(&mut router, runtime_state.clone());
        hcu0.try_accept(&mut router, runtime_state.clone());
    }
}

pub(super) async fn rx_network_1(
    interface: String,
    runtime_state: SharedOperandState,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;
    let mut router = Router::new(socket);

    let mut ems0 = EngineManagementSystem::new(
        crate::consts::J1939_ADDRESS_ENGINE0,
        crate::consts::J1939_ADDRESS_VMS,
    );

    loop {
        if let Err(e) = router.listen().await {
            log::error!("Failed to receive from router: {}", e);
        }

        ems0.try_accept(&mut router, runtime_state.clone());
    }
}

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
pub(super) async fn tx_network_0(
    interface: String,
    runtime_state: SharedOperandState,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;

    let name = glonax::j1939::NameBuilder::default()
        .identity_number(0x1)
        .manufacturer_code(J1939_NAME_MANUFACTURER_CODE)
        .function_instance(J1939_NAME_FUNCTION_INSTANCE)
        .ecu_instance(J1939_NAME_ECU_INSTANCE)
        .function(J1939_NAME_FUNCTION)
        .vehicle_system(J1939_NAME_VEHICLE_SYSTEM)
        .build();

    let _enc0 = KueblerEncoder::new(crate::consts::J1939_ADDRESS_ENCODER0);
    let _enc1 = KueblerEncoder::new(crate::consts::J1939_ADDRESS_ENCODER1);
    let _enc2 = KueblerEncoder::new(crate::consts::J1939_ADDRESS_ENCODER2);
    let _enc3 = KueblerEncoder::new(crate::consts::J1939_ADDRESS_ENCODER3);
    let _hcu0 = HydraulicControlUnit::new(
        crate::consts::J1939_ADDRESS_HCU0,
        crate::consts::J1939_ADDRESS_VMS,
    );

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(10));

    socket
        .send(&glonax::j1939::protocol::address_claimed(
            crate::consts::J1939_ADDRESS_VMS,
            name,
        ))
        .await?;

    loop {
        interval.tick().await;

        // if let Err(e) = socket.send_vectored(&hcu0.lock()).await {
        //     log::error!("Failed to send motion: {}", e);
        // } else {
        //     log::info!("Sent motion");
        // }
    }
}

// TODO: Move into runtime
pub(super) async fn tx_network_1(
    interface: String,
    runtime_state: SharedOperandState,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;

    let name = glonax::j1939::NameBuilder::default()
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

    let mut interval = tokio::time::interval(std::time::Duration::from_millis(10));

    socket
        .send(&glonax::j1939::protocol::address_claimed(
            crate::consts::J1939_ADDRESS_VMS,
            name,
        ))
        .await?;

    loop {
        interval.tick().await;

        {
            let rpm = runtime_state.read().await.state.engine_request;

            if let Err(e) = socket.send_vectored(&ems0.speed_request(rpm)).await {
                log::error!("Failed to send motion: {}", e);
            }
        }
    }
}
