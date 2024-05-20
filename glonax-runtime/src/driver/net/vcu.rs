use j1939::{protocol, Frame, Name, PGN};

use crate::net::Parsable;

use super::vecraft::{VecraftConfigMessage, VecraftStatusMessage};

const STATUS_PGN: u32 = 65_288;

pub enum VehicleMessage {
    VecraftConfig(VecraftConfigMessage),
    SoftwareIdentification((u8, u8, u8)),
    AddressClaim(Name),
    Status(VecraftStatusMessage),
}

pub struct VehicleControlUnit {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
}

impl VehicleControlUnit {
    /// Construct a new encoder service.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
        }
    }
}

impl Parsable<VehicleMessage> for VehicleControlUnit {
    fn parse(&mut self, frame: &Frame) -> Option<VehicleMessage> {
        if let Some(destination_address) = frame.id().destination_address() {
            if destination_address != self.destination_address && destination_address != 0xff {
                return None;
            }
        }

        match frame.id().pgn() {
            PGN::ProprietarilyConfigurableMessage1 => {
                if frame.pdu()[0..2] != [b'Z', b'C'] {
                    return None;
                }

                Some(VehicleMessage::VecraftConfig(
                    VecraftConfigMessage::from_frame(
                        self.destination_address,
                        self.source_address,
                        frame,
                    ),
                ))
            }
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

                        Some(VehicleMessage::SoftwareIdentification((
                            major, minor, patch,
                        )))
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

                Some(VehicleMessage::AddressClaim(Name::from_bytes(
                    frame.pdu().try_into().unwrap(),
                )))
            }
            PGN::ProprietaryB(STATUS_PGN) => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(VehicleMessage::Status(VecraftStatusMessage::from_frame(
                    frame,
                )))
            }
            _ => None,
        }
    }
}

impl super::J1939Unit for VehicleControlUnit {
    const VENDOR: &'static str = "laixer";
    const PRODUCT: &'static str = "vcu";

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    #[rustfmt::skip]
    async fn setup(
        &self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
    ) -> Result<(), super::J1939UnitError> {
        network.send(&protocol::request(self.destination_address, self.source_address, PGN::AddressClaimed)).await?;
        network.send(&protocol::request(self.destination_address, self.source_address, PGN::SoftwareIdentification)).await?;
        network.send(&protocol::request(self.destination_address, self.source_address, PGN::ComponentIdentification)).await?;
        ctx.tx_mark();

        Ok(())
    }

    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        let mut result = Result::<(), super::J1939UnitError>::Ok(());

        if ctx.is_rx_timeout(std::time::Duration::from_millis(1_000)) {
            result = Err(super::J1939UnitError::MessageTimeout);
        }

        if let Some(message) = network.try_accept(self) {
            match message {
                VehicleMessage::VecraftConfig(_config) => {}
                VehicleMessage::SoftwareIdentification(version) => {
                    ctx.rx_mark();

                    log::debug!(
                        "[xcan:0x{:X}] {}: Firmware version: {}.{}.{}",
                        self.destination(),
                        self.name(),
                        version.0,
                        version.1,
                        version.2
                    );
                }
                VehicleMessage::AddressClaim(name) => {
                    ctx.rx_mark();

                    log::debug!(
                        "[xcan:0x{:X}] {}: Address claimed: {}",
                        self.destination(),
                        self.name(),
                        name
                    );
                }
                VehicleMessage::Status(status) => {
                    ctx.rx_mark();

                    status.into_error()?;
                }
            }
        }

        result
    }
}
