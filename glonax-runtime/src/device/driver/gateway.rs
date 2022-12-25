use std::{sync::Arc, time::Duration};

use crate::{
    device::{self, CoreDevice},
    net::{ControlNet, Router},
};

const DEVICE_NET_LOCAL_ADDR: u8 = 0x9e;

pub struct Gateway {
    net: Arc<ControlNet>,
    router: Router,
    vecu: device::Vecu,
    mecu: device::Mecu,
    hcu: device::Hcu,
}

impl Gateway {
    /// Construct a new gateway device.
    pub fn new(name: &str, signal_manager: &crate::signal::SignalManager) -> std::io::Result<Self> {
        let net = Arc::new(ControlNet::new(name, DEVICE_NET_LOCAL_ADDR)?);

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
        if tokio::time::timeout(Duration::from_secs(1), self.router.listen())
            .await
            .is_err()
        {
            warn!("Network timeout: no incoming packets in last 1 second(s)")
        }

        self.router.try_accept(&mut self.vecu).await;
        self.router.try_accept(&mut self.mecu).await;
        self.router.try_accept(&mut self.hcu).await;

        Ok(())
    }
}
