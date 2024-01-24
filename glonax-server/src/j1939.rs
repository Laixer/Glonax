use glonax::runtime::SharedOperandState;

trait J1939Unit {
    fn try_accept(&mut self, router: &mut glonax::net::Router, runtime_state: SharedOperandState);
}

struct J1939UnitEncoder {
    encoder: glonax::device::KueblerEncoder,
}

impl J1939UnitEncoder {
    fn new(node: u8) -> Self {
        Self {
            encoder: glonax::device::KueblerEncoder::new(node),
        }
    }
}

impl J1939Unit for J1939UnitEncoder {
    fn try_accept(&mut self, router: &mut glonax::net::Router, runtime_state: SharedOperandState) {
        if let Some(message) = router.try_accept(&mut self.encoder) {
            if let Ok(mut runtime_state) = runtime_state.try_write() {
                runtime_state
                    .state
                    .encoders
                    .insert(message.node, message.position as f32);

                // TODO: Set the encoder state in the runtime state
                // if let Some(state) = message.state {
                //     log::debug!("0x{:X?} Encoder state: {:?}", message.node, state);
                // }
            }
        }
    }
}

#[derive(Default)]
struct J1939UnitEms {
    ems: glonax::net::EngineManagementSystem,
}

impl J1939Unit for J1939UnitEms {
    fn try_accept(&mut self, router: &mut glonax::net::Router, runtime_state: SharedOperandState) {
        if let Some(message) = router.try_accept(&mut self.ems) {
            if let Ok(mut runtime_state) = runtime_state.try_write() {
                runtime_state.state.engine.driver_demand = message.driver_demand.unwrap_or(0);
                runtime_state.state.engine.actual_engine = message.actual_engine.unwrap_or(0);
                runtime_state.state.engine.rpm = message.rpm.unwrap_or(0);
            }
        }
    }
}

pub(super) async fn network_0(
    interface: String,
    runtime_state: SharedOperandState,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let net = glonax::net::J1939Network::new(&interface, glonax::consts::DEFAULT_J1939_ADDRESS)?;
    let mut router = glonax::net::Router::new(net);

    let mut enc_0 = J1939UnitEncoder::new(0x6A);
    let mut enc_1 = J1939UnitEncoder::new(0x6B);
    let mut enc_2 = J1939UnitEncoder::new(0x6C);
    let mut enc_3 = J1939UnitEncoder::new(0x6D);

    loop {
        if let Err(e) = router.listen().await {
            log::error!("Failed to receive from router: {}", e);
        }

        enc_0.try_accept(&mut router, runtime_state.clone());
        enc_1.try_accept(&mut router, runtime_state.clone());
        enc_2.try_accept(&mut router, runtime_state.clone());
        enc_3.try_accept(&mut router, runtime_state.clone());
    }
}

pub(super) async fn network_1(
    interface: String,
    runtime_state: SharedOperandState,
) -> std::io::Result<()> {
    log::debug!("Starting J1939 service on {}", interface);

    let net = glonax::net::J1939Network::new(&interface, glonax::consts::DEFAULT_J1939_ADDRESS)?;
    let mut router = glonax::net::Router::new(net);

    let mut ems = J1939UnitEms::default();

    loop {
        if let Err(e) = router.listen().await {
            log::error!("Failed to receive from router: {}", e);
        }

        ems.try_accept(&mut router, runtime_state.clone());
    }
}
