use std::sync::Arc;

use glonax_j1939::{j1939, J1939Listener};

use crate::{
    device::{self, profile::CanDeviceRuleset, CoreDevice, Device, UserDevice},
    net::ControlNet2,
};

const DEVICE_NAME: &str = "gateway";
const DEVICE_NET_LOCAL_ADDR: u8 = 0x9e;

#[async_trait::async_trait]
pub trait GatewayClient: Send + Sync {
    fn from_net(net: Arc<ControlNet2<J1939Listener>>) -> Self
    where
        Self: Sized;

    async fn incoming(&mut self, frame: &j1939::Frame);
}

pub struct Gateway {
    sysname: String,
    net: Arc<ControlNet2<J1939Listener>>,
    last_interval: std::time::Instant,
    client_devices: Vec<device::DeviceDescriptor<dyn GatewayClient>>,
}

#[async_trait::async_trait]
impl UserDevice for Gateway {
    const NAME: &'static str = DEVICE_NAME;

    type DeviceRuleset = CanDeviceRuleset;

    #[inline]
    fn sysname(&self) -> &str {
        self.sysname.as_str()
    }

    #[inline]
    async fn from_sysname(name: &str) -> device::Result<Self> {
        Ok(Self::new(name))
    }
}

impl Gateway {
    pub fn new(name: &str) -> Self {
        let socket = J1939Listener::bind(name, DEVICE_NET_LOCAL_ADDR).unwrap();
        socket.set_broadcast(true).unwrap();

        Self {
            sysname: name.to_owned(),
            net: Arc::new(ControlNet2::new(socket)),
            last_interval: std::time::Instant::now(),
            client_devices: vec![],
        }
    }

    // TODO: FIX
    pub async fn wait_online(&self) -> bool {
        let _ = self.net.accept().await;
        true
    }

    pub fn new_gateway_device<T>(&mut self) -> T
    where
        T: Device + GatewayClient + 'static,
    {
        T::from_net(self.net.clone())
    }

    pub fn subscribe(&mut self, device: device::DeviceDescriptor<dyn GatewayClient>) {
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
        if self.last_interval.elapsed() >= std::time::Duration::from_secs(1) {
            trace!("Announce status");
            self.net.announce_status().await;
            self.last_interval = std::time::Instant::now();
        }

        let frame = self.net.accept().await;
        for device in self.client_devices.iter_mut() {
            device.lock().await.incoming(&frame).await;
        }

        Ok(())
    }
}
