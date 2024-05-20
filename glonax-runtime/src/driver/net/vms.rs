use j1939::{protocol, spn, Frame, FrameBuilder, IdBuilder, PGN};

use crate::net::Parsable;

#[derive(Default)]
pub struct VehicleManagementSystem {
    /// Source address.
    source_address: u8,
}

impl VehicleManagementSystem {
    pub fn new(sa: u8) -> Self {
        Self { source_address: sa }
    }
}

impl Parsable<PGN> for VehicleManagementSystem {
    fn parse(&mut self, frame: &Frame) -> Option<PGN> {
        if frame.id().pgn() == PGN::Request {
            if frame.id().destination_address() != Some(self.source_address) {
                return None;
            }

            Some(protocol::request_from_pdu(frame.pdu()))
        } else {
            None
        }
    }
}

impl super::J1939Unit for VehicleManagementSystem {
    const VENDOR: &'static str = "laixer";
    const PRODUCT: &'static str = "vms";

    fn destination(&self) -> u8 {
        self.source_address
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
        network.send(&protocol::address_claimed(self.source_address, network.name())).await?;
        ctx.tx_mark();

        Ok(())
    }

    async fn try_accept(
        &mut self,
        _ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        if let Some(pgn) = network.try_accept(self) {
            match pgn {
                #[rustfmt::skip]
                PGN::AddressClaimed => {
                    network.send(&protocol::address_claimed(self.source_address, network.name())).await?;
                }
                PGN::SoftwareIdentification => {
                    let id = IdBuilder::from_pgn(PGN::SoftwareIdentification)
                        .sa(self.source_address)
                        .build();

                    // TODO: Move this to consts
                    let version_major: u8 = crate::consts::VERSION_MAJOR.parse().unwrap();
                    let version_minor: u8 = crate::consts::VERSION_MINOR.parse().unwrap();
                    let version_patch: u8 = crate::consts::VERSION_PATCH.parse().unwrap();

                    let frame = FrameBuilder::new(id)
                        .copy_from_slice(&[1, version_major, version_minor, version_patch, b'*'])
                        .build();

                    network.send(&frame).await?;
                }
                PGN::TimeDate => {
                    use chrono::prelude::*;

                    let utc = chrono::Utc::now();
                    let timedate = spn::TimeDate {
                        year: utc.year(),
                        month: utc.month(),
                        day: utc.day(),
                        hour: utc.hour(),
                        minute: utc.minute(),
                        second: utc.second(),
                    };

                    let id = IdBuilder::from_pgn(PGN::TimeDate)
                        .sa(self.source_address)
                        .build();

                    let frame = FrameBuilder::new(id)
                        .copy_from_slice(&timedate.to_pdu())
                        .build();

                    network.send(&frame).await?;
                }
                _ => (),
            }
        }

        Ok(())
    }
}
