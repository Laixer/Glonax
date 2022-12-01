use std::{collections::HashMap, sync::Arc};

use glonax_j1939::Frame;

use crate::{device::Device, net::ControlNet};

const DEVICE_NAME: &str = "v-ecu";

pub struct Vecu {
    node_list: HashMap<u8, u8>,
}

impl Device for Vecu {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl super::gateway::GatewayClient for Vecu {
    fn from_net(_net: Arc<ControlNet>) -> Self {
        Self {
            node_list: HashMap::new(),
        }
    }

    async fn incoming(&mut self, frame: &Frame) {
        if self
            .node_list
            .insert(frame.id().sa(), frame.pdu()[1])
            .is_none()
        {
            debug!("New node on network: 0x{:X?}", frame.id().sa());
        }
    }
}
