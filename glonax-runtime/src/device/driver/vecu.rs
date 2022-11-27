use std::{collections::HashMap, sync::Arc};

use glonax_j1939::Frame;

use crate::{
    device::Device,
    net::{ControlNet, StatusService},
};

const DEVICE_NAME: &str = "v-ecu";

pub struct Vecu {
    status_serivce: StatusService,
    node_list: HashMap<u8, u8>,
}

impl Device for Vecu {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl super::gateway::GatewayClient for Vecu {
    fn from_net(net: Arc<ControlNet>) -> Self {
        Self {
            status_serivce: StatusService::new(net, 0x0), // TODO: Find an address.
            node_list: HashMap::new(),
        }
    }

    async fn incoming(&mut self, frame: &Frame) {
        // TODO: Need an external trigger.
        self.status_serivce.interval().await;

        if frame.id().pgn() == 65_282 {
            let state = match crate::net::spn_state(frame.pdu()[1]) {
                Some(crate::net::State::Nominal) => Some("nominal"),
                Some(crate::net::State::Ident) => Some("ident"),
                Some(crate::net::State::Faulty) => Some("faulty"),
                _ => None,
            };

            let firmware_version =
                crate::net::spn_firmware_version(frame.pdu()[2..5].try_into().unwrap());

            let last_error = crate::net::spn_last_error(frame.pdu()[6..8].try_into().unwrap());

            trace!(
                "0x{:X?} State: {}; Version: {}; Last error: {}",
                frame.id().sa(),
                state.map_or_else(|| "-".to_owned(), |f| { f.to_string() }),
                firmware_version.map_or_else(
                    || "-".to_owned(),
                    |f| { format!("{}.{}.{}", f.0, f.1, f.2) }
                ),
                last_error.map_or_else(|| "-".to_owned(), |f| { f.to_string() })
            );

            if self
                .node_list
                .insert(frame.id().sa(), frame.pdu()[1])
                .is_none()
            {
                info!("New node on network: 0x{:X?}", frame.id().sa());
            }
        }
    }
}
