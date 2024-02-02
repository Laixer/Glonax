use j1939::{decode, Frame, Name, PGN};

use crate::net::Parsable;

pub enum J1939Message {
    /// Software identification.
    SoftwareIndent((u8, u8, u8)),
    /// Requested PGN.
    RequestPGN(u32),
    /// Address claim.
    AddressClaim(Name),
    /// Acknowledged.
    Acknowledged(u8),
    /// Time and date.
    TimeDate(chrono::DateTime<chrono::Utc>),
    /// Proprietary B.
    ProprietaryB([u8; 8]),
}

impl J1939Message {
    pub fn from_frame(_: u8, frame: &Frame) -> Option<Self> {
        match frame.id().pgn() {
            PGN::SoftwareIdentification => {
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

                        Some(Self::SoftwareIndent((major, minor, patch)))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            PGN::Request => Some(Self::RequestPGN(u32::from_be_bytes([
                0x0,
                frame.pdu()[2],
                frame.pdu()[1],
                frame.pdu()[0],
            ]))),
            PGN::AddressClaimed => Some(Self::AddressClaim(Name::from_bytes(
                frame.pdu().try_into().unwrap(),
            ))),
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

#[derive(Default)]
pub struct J1939ApplicationInspector;

impl Parsable<J1939Message> for J1939ApplicationInspector {
    fn parse(&mut self, frame: &Frame) -> Option<J1939Message> {
        J1939Message::from_frame(0x0, frame)
    }
}
