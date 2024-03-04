use j1939::{protocol, spn, Frame, FrameBuilder, IdBuilder, NameBuilder, PGN};

use crate::net::Parsable;

// TODO: Get from configuration.

/// J1939 name manufacturer code.
const J1939_NAME_MANUFACTURER_CODE: u16 = 0x717;
/// J1939 name function instance.
const J1939_NAME_FUNCTION_INSTANCE: u8 = 6;
/// J1939 name ECU instance.
const J1939_NAME_ECU_INSTANCE: u8 = 0;
/// J1939 name function.
const J1939_NAME_FUNCTION: u8 = 0x1C;
/// J1939 name vehicle system.
const J1939_NAME_VEHICLE_SYSTEM: u8 = 2;

#[derive(Default)]
pub struct RequestResponder {
    /// Source address.
    source_address: u8,
}

impl RequestResponder {
    pub fn new(sa: u8) -> Self {
        Self { source_address: sa }
    }
}

impl Parsable<PGN> for RequestResponder {
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

impl super::J1939Unit for RequestResponder {
    async fn try_accept(
        &mut self,
        router: &crate::net::Router,
        _runtime_state: crate::runtime::SharedOperandState,
    ) {
        if let Some(pgn) = router.try_accept(self) {
            match pgn {
                PGN::AddressClaimed => {
                    let name = NameBuilder::default()
                        .identity_number(0x1)
                        .manufacturer_code(J1939_NAME_MANUFACTURER_CODE)
                        .function_instance(J1939_NAME_FUNCTION_INSTANCE)
                        .ecu_instance(J1939_NAME_ECU_INSTANCE)
                        .function(J1939_NAME_FUNCTION)
                        .vehicle_system(J1939_NAME_VEHICLE_SYSTEM)
                        .build();

                    if let Err(e) = router
                        .inner()
                        .send(&protocol::address_claimed(self.source_address, name))
                        .await
                    {
                        log::error!("Failed to send address claimed: {}", e);
                    }
                }
                PGN::SoftwareIdentification => {
                    let id = IdBuilder::from_pgn(PGN::SoftwareIdentification)
                        .sa(self.source_address)
                        .build();

                    let version_major: u8 = crate::consts::VERSION_MAJOR.parse().unwrap();
                    let version_minor: u8 = crate::consts::VERSION_MINOR.parse().unwrap();
                    let version_patch: u8 = crate::consts::VERSION_PATCH.parse().unwrap();

                    let frame = FrameBuilder::new(id)
                        .copy_from_slice(&[1, version_major, version_minor, version_patch, b'*'])
                        .build();

                    if let Err(e) = router.inner().send(&frame).await {
                        log::error!("Failed to send software identification: {}", e);
                    }
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

                    if let Err(e) = router.inner().send(&frame).await {
                        log::error!("Failed to send time date: {}", e);
                    }
                }
                _ => (),
            }
        }
    }
}
