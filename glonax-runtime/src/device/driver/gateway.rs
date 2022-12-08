use std::sync::Arc;

use crate::{
    device::{self, CoreDevice, Device},
    net::{ControlNet, Router},
};

const DEVICE_NAME: &str = "gateway";
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

        let vecu = device::Vecu::new(net.clone(), signal_manager.publisher());
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

    /// Wait until the network comes online.
    ///
    /// The controller accepts the first ingress frame as evidence of
    /// an online and operational network. If the network does not connect
    /// to any nodes the network can still be operational but is not
    /// considered online.
    #[inline]
    pub async fn wait_online(&self) -> bool {
        self.net.accept().await.is_ok()
    }

    pub fn hcu(&self) -> device::Hcu {
        device::Hcu::new(self.net.clone())
    }
}

impl Device for Gateway {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl CoreDevice for Gateway {
    async fn next(&mut self) -> device::Result<()> {
        self.router.accept().await.unwrap(); // TODO: Handle err.

        self.router.try_accept(&mut self.vecu).await;
        self.router.try_accept(&mut self.mecu).await;
        self.router.try_accept(&mut self.hcu).await;

        Ok(())
    }
}
