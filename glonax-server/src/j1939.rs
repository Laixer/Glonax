use std::time::Duration;

use glonax::runtime::SharedOperandState;

use glonax::driver::net::J1939Unit;
use glonax::driver::{EngineManagementSystem, HydraulicControlUnit};
use glonax::net::{CANSocket, Router, SockAddrCAN};

// TODO: Move into runtime
pub(super) async fn atx_network_1(
    interface: String,
    runtime_state: SharedOperandState,
    mut motion_rx: crate::device::MotionReceiver,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 ATX service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;
    let router = Router::new(socket);

    let hcu0 = HydraulicControlUnit::new(
        crate::consts::J1939_ADDRESS_HCU0,
        crate::consts::J1939_ADDRESS_VMS,
    );

    while let Some(motion) = motion_rx.recv().await {
        runtime_state.write().await.state.motion = motion.clone();

        hcu0.tick(&router, runtime_state.clone()).await;
    }

    Ok(())
}

// TODO: Move into runtime
pub(super) async fn tx_network_1(
    interface: String,
    runtime_state: SharedOperandState,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 TX service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;
    let router = Router::new(socket);

    let ems0 = EngineManagementSystem::new(
        crate::consts::J1939_ADDRESS_ENGINE0,
        crate::consts::J1939_ADDRESS_VMS,
    );
    let hcu0 = HydraulicControlUnit::new(
        crate::consts::J1939_ADDRESS_HCU0,
        crate::consts::J1939_ADDRESS_VMS,
    );

    let mut interval = tokio::time::interval(Duration::from_millis(10));

    while shutdown.is_empty() {
        interval.tick().await;

        ems0.tick(&router, runtime_state.clone()).await;
        hcu0.tick(&router, runtime_state.clone()).await;
    }

    Ok(())
}
