use glonax::runtime::SharedOperandState;

use glonax::consts;
use glonax::device::net::J1939Unit;
use glonax::net::{J1939Network, Router};

// TODO: Move into runtime
pub(super) async fn network_0(
    interface: String,
    runtime_state: SharedOperandState,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let net = J1939Network::new(&interface, consts::DEFAULT_J1939_ADDRESS)?;
    let mut router = Router::new(net);

    let mut enc0 = glonax::device::KueblerEncoder::new(0x6A);
    let mut enc1 = glonax::device::KueblerEncoder::new(0x6B);
    let mut enc2 = glonax::device::KueblerEncoder::new(0x6C);
    let mut enc3 = glonax::device::KueblerEncoder::new(0x6D);
    let mut hcu0 = glonax::device::HydraulicControlUnit::new(0x4A);

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

    let net = J1939Network::new(&interface, consts::DEFAULT_J1939_ADDRESS)?;
    let mut router = Router::new(net);

    let mut ems0 = glonax::device::EngineManagementSystem;

    loop {
        if let Err(e) = router.listen().await {
            log::error!("Failed to receive from router: {}", e);
        }

        ems0.try_accept(&mut router, runtime_state.clone());
    }
}
