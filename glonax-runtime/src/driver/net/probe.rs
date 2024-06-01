use j1939::{protocol, Frame, PGN};

use crate::net::{ControlNetwork, Parsable};

#[derive(Debug, Clone)]
pub struct EcuAddress {
    /// Destination address.
    pub destination_address: Option<u8>,
    /// Source address.
    pub source_address: Option<u8>,
}

#[derive(Default)]
pub struct Probe {
    /// ECU addresses.
    addresses: Vec<u8>,
}

impl Probe {
    #[rustfmt::skip]
    pub async fn send_probe(&self, da: u8, sa: u8, network: &mut ControlNetwork) -> std::io::Result<()> {
        network.send(&protocol::request(da, sa, PGN::AddressClaimed)).await?;
        network.send(&protocol::request(da, sa, PGN::SoftwareIdentification)).await?;
        network.send(&protocol::request(da, sa, PGN::ComponentIdentification)).await?;
        network.send(&protocol::request(da, sa, PGN::VehicleIdentification)).await?;
        network.send(&protocol::request(da, sa, PGN::TimeDate)).await?;

        Ok(())
    }
}

impl Parsable<EcuAddress> for Probe {
    fn parse(&self, frame: &Frame) -> Option<EcuAddress> {
        let mut address = EcuAddress {
            destination_address: None,
            source_address: None,
        };

        if !self.addresses.contains(&frame.id().source_address()) {
            address.source_address = Some(frame.id().source_address());
            // TODO: Parse should not modify state.
            // self.addresses.push(frame.id().source_address());
        }

        if let Some(destination_address) = frame.id().destination_address() {
            if destination_address != 0xff && !self.addresses.contains(&destination_address) {
                address.destination_address = Some(destination_address);
                // TODO: Parse should not modify state.
                // self.addresses.push(destination_address);
            }
        }

        None
    }
}
