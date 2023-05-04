use glonax_j1939::*;

use super::Parsable;

pub struct J1939ApplicationInspector {}

pub struct J1939Message {
    pub software_indent: Option<(u8, u8, u8)>,
    pub request_pgn: Option<u32>,
    pub address_claim: Option<(u8, u8)>,
    pub acknowledged: Option<u8>,
}

impl J1939Message {
    pub fn from_frame(_: u8, frame: &Frame) -> Self {
        let mut software_indent = None;
        let mut request_pgn = None;
        let mut address_claim = None;
        let mut acknowledged = None;

        match frame.id().pgn() {
            PGN::SoftwareIdentification => {
                let mut major = 0;
                let mut minor = 0;
                let mut patch = 0;

                if frame.pdu()[3] != 0xff {
                    major = frame.pdu()[3];
                }
                if frame.pdu()[4] != 0xff {
                    minor = frame.pdu()[4];
                }
                if frame.pdu()[5] != 0xff {
                    patch = frame.pdu()[5];
                }

                software_indent = Some((major, minor, patch));
            }
            PGN::Request => {
                request_pgn = Some(u32::from_be_bytes([
                    0x0,
                    frame.pdu()[2],
                    frame.pdu()[1],
                    frame.pdu()[0],
                ]));
            }
            PGN::AddressClaimed => {
                let function = frame.pdu()[5];
                let arbitrary_address = frame.pdu()[7] >> 7;

                address_claim = Some((function, arbitrary_address));
            }
            PGN::AcknowledgmentMessage => {
                acknowledged = Some(frame.pdu()[0]);
            }
            _ => {}
        }

        Self {
            software_indent,
            request_pgn,
            address_claim,
            acknowledged,
        }
    }
}

impl Parsable<J1939Message> for J1939ApplicationInspector {
    fn parse(&mut self, frame: &Frame) -> Option<J1939Message> {
        Some(J1939Message::from_frame(0x0, frame))
    }
}

impl J1939ApplicationInspector {
    pub fn new() -> Self {
        Self {}
    }
}
