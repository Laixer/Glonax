use glonax::runtime::SharedOperandState;

use glonax::driver::net::J1939Unit;
use glonax::driver::HydraulicControlUnit;
use glonax::net::{CANSocket, Router, SockAddrCAN};

/// Vehicle Management System J1939 address.
const J1939_ADDRESS_VMS: u8 = 0x27;
/// Hydraulic Control Unit 0 J1939 address.
const J1939_ADDRESS_HCU0: u8 = 0x4a;

// TODO: Move into runtime
pub(super) async fn atx_network_1(
    interface: String,
    runtime_state: SharedOperandState,
    mut motion_rx: crate::device::MotionReceiver,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 ATX service on {}", interface);

    let socket = CANSocket::bind(&SockAddrCAN::new(&interface))?;
    let router = Router::new(socket);

    let hcu0 = HydraulicControlUnit::new(J1939_ADDRESS_HCU0, J1939_ADDRESS_VMS);

    while let Some(motion) = motion_rx.recv().await {
        runtime_state.write().await.state.motion = motion.clone();

        hcu0.tick(&router, runtime_state.clone()).await;
    }

    Ok(())
}
