use j1939::{protocol, Frame, Name, PGN};

use crate::{
    core::Object,
    net::Parsable,
    runtime::{J1939Unit, J1939UnitError, NetDriverContext},
};

pub enum ECUMessage {
    SoftwareIdentification((u8, u8, u8)),
    AddressClaim(Name),
}

#[derive(Clone)]
pub struct ElectronicControlUnit {
    /// Network interface.
    interface: String,
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
}

impl ElectronicControlUnit {
    pub fn new(interface: &str, da: u8, sa: u8) -> Self {
        Self {
            interface: interface.to_string(),
            destination_address: da,
            source_address: sa,
        }
    }
}

impl Parsable<ECUMessage> for ElectronicControlUnit {
    fn parse(&self, frame: &Frame) -> Option<ECUMessage> {
        if let Some(destination_address) = frame.id().destination_address() {
            if destination_address != self.destination_address && destination_address != 0xff {
                return None;
            }
        }

        match frame.id().pgn() {
            PGN::SoftwareIdentification => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                let fields = frame.pdu()[0];

                if fields >= 1 {
                    if frame.pdu()[4] == b'*' {
                        let mut major = 0;
                        let mut minor = 0;
                        let mut patch = 0;

                        if frame.pdu()[1] != 0xff {
                            major = frame.pdu()[1];
                        }
                        if frame.pdu()[2] != 0xff {
                            minor = frame.pdu()[2];
                        }
                        if frame.pdu()[3] != 0xff {
                            patch = frame.pdu()[3];
                        }

                        Some(ECUMessage::SoftwareIdentification((major, minor, patch)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            PGN::AddressClaimed => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(ECUMessage::AddressClaim(Name::from_bytes(
                    frame.pdu().try_into().unwrap(),
                )))
            }
            _ => None,
        }
    }
}

impl J1939Unit for ElectronicControlUnit {
    fn vendor(&self) -> &'static str {
        "j1939"
    }

    fn product(&self) -> &'static str {
        "ecu"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    fn setup(
        &self,
        _ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::AddressClaimed,
        ));
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::SoftwareIdentification,
        ));
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::ComponentIdentification,
        ));

        Ok(())
    }

    fn try_recv(
        &self,
        ctx: &mut NetDriverContext,
        frame: &j1939::Frame,
        _rx_queue: &mut Vec<Object>,
    ) -> Result<(), J1939UnitError> {
        if let Some(message) = self.parse(frame) {
            match message {
                ECUMessage::SoftwareIdentification(version) => {
                    debug!(
                        "[{}] {}: Firmware version: {}.{}.{}",
                        self.interface,
                        self.name(),
                        version.0,
                        version.1,
                        version.2
                    );

                    ctx.rx_mark();

                    return Ok(());
                }
                ECUMessage::AddressClaim(name) => {
                    debug!(
                        "[{}] {}: Address claimed: {}",
                        self.interface,
                        self.name(),
                        name
                    );

                    ctx.rx_mark();

                    return Ok(());
                }
            }
        }

        Ok(())
    }
}
