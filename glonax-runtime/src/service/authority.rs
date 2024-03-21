use std::path::PathBuf;

use crate::runtime::{Service, SharedOperandState};

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct GnssConfig {
    /// Path to the serial device
    pub device: PathBuf,
    /// Baud rate of the serial device
    pub baud_rate: usize,
}

pub struct NetworkAuthority {
    router: crate::net::Router,
    // line_reader: Lines<BufReader<Uart>>,
    network: crate::runtime::ControlNetwork,
    path: PathBuf,
}

impl Service<GnssConfig> for NetworkAuthority {
    fn new(config: GnssConfig) -> Self
    where
        Self: Sized,
    {
        let interface = "vcan0";
        let socket = crate::net::CANSocket::bind(&crate::net::SockAddrCAN::new(&interface)).unwrap();
        let router = crate::net::Router::new(socket);

        let network = crate::runtime::ControlNetwork::new(0x27);

        Self {
            router,
            network,
            path: config.device,
        }
    }

    fn ctx(&self) -> crate::runtime::ServiceContext {
        crate::runtime::ServiceContext::new("authority", Some(self.path.display().to_string()))
    }

    async fn wait_io(&mut self, runtime_state: SharedOperandState) {
        use crate::driver::net::J1939Unit;

        self.router.send(&j1939::protocol::address_claimed(self.router.source_address(), *self.router.name())).await.unwrap();

        loop {
            if let Err(e) = self.router.listen().await {
                log::error!("Failed to receive from router: {}", e);
            }

            let state = crate::driver::net::J1939UnitOperationState::Running;
            for (drv, ctx) in self.network.network.iter_mut() {
                //
                //
                match drv {
                    crate::driver::net::NetDriver::KueblerEncoder(enc) => {
                        if let Err(error) = enc.try_accept(ctx, &state, &self.router, runtime_state.clone()).await {
                            log::error!("Failed to accept message: {}", error);
                        }
                    }
                    crate::driver::net::NetDriver::KueblerInclinometer(imu) => {
                        if let Err(error) = imu.try_accept(ctx, &state, &self.router, runtime_state.clone()).await {
                            log::error!("Failed to accept message: {}", error);
                        }
                    }
                    crate::driver::net::NetDriver::VolvoD7E(ems) => {
                        if let Err(error) = ems.try_accept(ctx, &state, &self.router, runtime_state.clone()).await {
                            log::error!("Failed to accept message: {}", error);
                        }
                    }
                    crate::driver::net::NetDriver::BoschEngineManagementSystem(ems) => {
                        if let Err(error) = ems.try_accept(ctx, &state, &self.router, runtime_state.clone()).await {
                            log::error!("Failed to accept message: {}", error);
                        }
                    }
                    crate::driver::net::NetDriver::HydraulicControlUnit(hcu) => {
                        if let Err(error) = hcu.try_accept(ctx, &state, &self.router, runtime_state.clone()).await {
                            log::error!("Failed to accept message: {}", error);
                        }
                    }
                    crate::driver::net::NetDriver::RequestResponder(rrp) => {
                        if let Err(error) = rrp.try_accept(ctx, &state, &self.router, runtime_state.clone()).await {
                            log::error!("Failed to accept message: {}", error);
                        }
                    }
                    crate::driver::net::NetDriver::VehicleControlUnit(vcu) => {
                        if let Err(error) = vcu.try_accept(ctx, &state, &self.router, runtime_state.clone()).await {
                            log::error!("Failed to accept message: {}", error);
                        }
                    }
                }
                //
                //
            }
        }
    }
}
