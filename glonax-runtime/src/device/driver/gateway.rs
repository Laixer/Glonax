use std::sync::Arc;

use glonax_j1939::Frame;

use crate::{
    device::{self, CoreDevice, Device},
    net::ControlNet,
};

const DEVICE_NAME: &str = "gateway";
const DEVICE_NET_LOCAL_ADDR: u8 = 0x9e;

#[async_trait::async_trait]
pub trait GatewayClient: Send + Sync {
    fn from_net(net: Arc<ControlNet>) -> Self
    where
        Self: Sized;

    async fn incoming(&mut self, frame: &Frame);
}

pub struct Gateway {
    net: Arc<ControlNet>,
    client_devices: Vec<Box<dyn GatewayClient>>,
}

impl Gateway {
    /// Construct a new gateway device.
    pub fn new(name: &str) -> Self {
        Self {
            net: Arc::new(ControlNet::new(name, DEVICE_NET_LOCAL_ADDR).unwrap()),
            client_devices: vec![],
        }
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

    pub fn new_gateway_device<T>(&mut self) -> T
    where
        T: Device + GatewayClient + 'static,
    {
        self.subscribe(Box::new(T::from_net(self.net.clone())));

        T::from_net(self.net.clone())
    }

    pub fn subscribe(&mut self, device: Box<dyn GatewayClient>) {
        trace!("Subscribe new device to gateway");

        self.client_devices.push(device);
    }
}

unsafe impl Send for Gateway {}

impl Device for Gateway {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl CoreDevice for Gateway {
    async fn next(&mut self) -> device::Result<()> {
        // TODO: Wrap error in device result.
        let frame = self.net.accept().await.unwrap();
        for device in self.client_devices.iter_mut() {
            device.incoming(&frame).await
        }

        Ok(())
    }
}
