use std::sync::Arc;

use glonax_j1939::J1939Listener;

use crate::{device::Device, net::ControlNet2};

const DEVICE_NAME: &str = "v-ecu";
// const DEVICE_NET_HCU_ADDR: u8 = 0xee;

pub struct Vecu(Arc<ControlNet2<J1939Listener>>);

unsafe impl Send for Vecu {}

impl Device for Vecu {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl super::gateway::GatewayClient for Vecu {
    fn from_net(net: Arc<ControlNet2<J1939Listener>>) -> Self {
        Self(net)
    }

    async fn incoming(&mut self, frame: &glonax_j1939::j1939::Frame) {
        if frame.id().pgn() == 65_282 {
            let state = match frame.pdu()[1] {
                1 => "boot0",
                5 => "init core peripherals",
                6 => "init auxiliary modules",
                20 => "nominal",
                255 => "faulty",
                _ => "other",
            };

            trace!(
                "0x{:X?} State: {}; Last error: {}",
                frame.id().sa(),
                state,
                u16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap())
            );
        }
    }
}
