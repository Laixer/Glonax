use glonax::runtime::SharedOperandState;

use glonax::device::net::J1939Unit;
use glonax::device::{EngineManagementSystem, HydraulicControlUnit, KueblerEncoder};
use glonax::net::{CANSocket, Router, SockAddrCAN};

// TODO: Move into runtime
pub(super) async fn network_0(
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

pub(super) async fn network_1(
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
