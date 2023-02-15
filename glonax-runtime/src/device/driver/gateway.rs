use std::sync::Arc;

use crate::{
    device::{self, CoreDevice},
    net::{J1939Network, Router},
};

const DEVICE_NET_LOCAL_ADDR: u8 = 0x9e;

pub struct Gateway {
    net: Arc<J1939Network>,
    router: Router,
    vecu: device::Vecu,
    mecu: device::Mecu,
    hcu: device::Hcu,
}

impl Gateway {
    /// Construct a new gateway device.
    pub fn new(name: &str, signal_manager: &crate::signal::SignalManager) -> std::io::Result<Self> {
        let net = Arc::new(J1939Network::new(name, DEVICE_NET_LOCAL_ADDR)?);

        let vecu = device::Vecu::new(signal_manager.publisher());
        let mecu = device::Mecu::new(net.clone(), signal_manager.publisher());
        let hcu = device::Hcu::new(net.clone());

        Ok(Self {
            net: net.clone(),
            router: Router::new(net),
            vecu,
            mecu,
            hcu,
        })
    }

    pub fn hcu(&self) -> device::Hcu {
        device::Hcu::new(self.net.clone())
    }
}

#[async_trait::async_trait]
impl CoreDevice for Gateway {
    async fn next(&mut self) -> device::Result<()> {
        self.router.try_accept(&mut self.vecu);
        self.router.try_accept(&mut self.mecu);
        self.router.try_accept(&mut self.hcu);

        Ok(())
    }
}
