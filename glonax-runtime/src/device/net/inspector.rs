use j1939::{decode, Frame, PGN};

use crate::net::Parsable;

#[derive(Default)]
pub struct J1939ApplicationInspector;

pub enum J1939Message {
    /// Software identification.
    SoftwareIndent((u8, u8, u8)),
    /// Requested PGN.
    RequestPGN(u32),
    /// Address claim.
    AddressClaim((u8, u8)),
    /// Acknowledged.
    Acknowledged(u8),
    /// Time and date.
    TimeDate(chrono::DateTime<chrono::Utc>),
    /// Proprietary B.
    ProprietaryB([u8; 8]),
}

impl J1939Message {
    pub fn from_frame(_: u8, frame: &Frame) -> Option<Self> {
        // TODO: Move most of the logic to j1939 crate
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

                Some(Self::SoftwareIndent((major, minor, patch)))
            }
            PGN::Request => Some(Self::RequestPGN(u32::from_be_bytes([
                0x0,
                frame.pdu()[2],
                frame.pdu()[1],
                frame.pdu()[0],
            ]))),
            PGN::AddressClaimed => {
                let function = frame.pdu()[5];
                let arbitrary_address = frame.pdu()[7] >> 7;

                Some(Self::AddressClaim((function, arbitrary_address)))
            }
            PGN::AcknowledgmentMessage => Some(Self::Acknowledged(frame.pdu()[0])),
            PGN::TimeDate => {
                use chrono::{TimeZone, Utc};

                let dt = Utc.with_ymd_and_hms(
                    decode::spn964(frame.pdu()[5]).unwrap_or(0) as i32,
                    decode::spn963(frame.pdu()[3]).unwrap_or(0) as u32,
                    decode::spn962(frame.pdu()[4]).unwrap_or(0) as u32,
                    decode::spn961(frame.pdu()[2]).unwrap_or(0) as u32,
                    decode::spn960(frame.pdu()[1]).unwrap_or(0) as u32,
                    decode::spn959(frame.pdu()[0]).unwrap_or(0) as u32,
                );

                Some(Self::TimeDate(dt.single().unwrap()))
            }
            PGN::ProprietaryB(_) => {
                let mut data = [0; 8];
                data.copy_from_slice(&frame.pdu()[0..8]);

                Some(Self::ProprietaryB(data))
            }
            _ => None,
        }
    }
}

impl Parsable<J1939Message> for J1939ApplicationInspector {
    fn parse(&mut self, frame: &Frame) -> Option<J1939Message> {
        J1939Message::from_frame(0x0, frame)
    }
}
