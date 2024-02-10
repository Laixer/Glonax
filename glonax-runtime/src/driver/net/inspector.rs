use j1939::{protocol, Frame, Name, PGN};

use crate::net::Parsable;

pub enum J1939Message {
    /// Software identification.
    SoftwareIndent((u8, u8, u8)),
    /// Requested PGN.
    RequestPGN(PGN),
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
    pub fn from_frame(frame: &Frame) -> Option<Self> {
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
            // TODO: Maybe use the request responder
            PGN::Request => Some(Self::RequestPGN(protocol::request_from_pdu(frame.pdu()))),
            PGN::AddressClaimed => Some(Self::AddressClaim(Name::from_bytes(
                frame.pdu().try_into().unwrap(),
            ))),
            PGN::AcknowledgmentMessage => Some(Self::Acknowledged(frame.pdu()[0])),
            PGN::TimeDate => {
                use chrono::{TimeZone, Utc};

                let timedate = j1939::spn::TimeDate::from_pdu(frame.pdu());

                let dt = Utc.with_ymd_and_hms(
                    timedate.year,
                    timedate.month,
                    timedate.day,
                    timedate.hour,
                    timedate.minute,
                    timedate.second,
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
        J1939Message::from_frame(frame)
    }
}
