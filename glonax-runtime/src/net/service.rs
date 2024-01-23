use glonax_j1939::*;

use super::Parsable;

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
    pub fn from_frame(_: u8, frame: &Frame) -> Self {
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

                Self::SoftwareIndent((major, minor, patch))
            }
            PGN::Request => Self::RequestPGN(u32::from_be_bytes([
                0x0,
                frame.pdu()[2],
                frame.pdu()[1],
                frame.pdu()[0],
            ])),
            PGN::AddressClaimed => {
                let function = frame.pdu()[5];
                let arbitrary_address = frame.pdu()[7] >> 7;

                Self::AddressClaim((function, arbitrary_address))
            }
            PGN::AcknowledgmentMessage => Self::Acknowledged(frame.pdu()[0]),
            PGN::TimeDate => {
                use chrono::{TimeZone, Utc};

                // println!("Seconds: {}", frame.pdu()[0] as f32 * 0.25); // SPN 959
                // println!("Minutes: {}", frame.pdu()[1]); // SPN 960
                // println!("Hours: {}", frame.pdu()[2]); // SPN 961
                // println!("Month: {}", frame.pdu()[3]); // SPN 963
                // println!("Day: {}", frame.pdu()[4] as f32 * 0.25); // SPN 962
                // println!("Year: {}", frame.pdu()[5] as u16 + 1985); // SPN 964

                // if frame.pdu()[6] != 0xff {
                //     println!("Local minute offset: {}", frame.pdu()[6] - 125); // SPN 1601
                // }
                // if frame.pdu()[7] != 0xff {
                //     println!("Local hour offset: {}", frame.pdu()[7] as i8 - 125);
                //     // SPN 1602
                // }

                let dt = Utc.with_ymd_and_hms(
                    frame.pdu()[5] as i32 + 1985,
                    frame.pdu()[3] as u32,
                    (frame.pdu()[4] as f32 * 0.25) as u32,
                    frame.pdu()[2] as u32,
                    frame.pdu()[1] as u32,
                    (frame.pdu()[0] as f32 * 0.25) as u32,
                );

                Self::TimeDate(dt.single().unwrap())
            }
            PGN::ProprietaryB(_) => {
                let mut data = [0; 8];
                data.copy_from_slice(&frame.pdu()[0..8]);

                Self::ProprietaryB(data)
            }
            _ => todo!(),
        }
    }
}

impl Parsable<J1939Message> for J1939ApplicationInspector {
    fn parse(&mut self, frame: &Frame) -> Option<J1939Message> {
        Some(J1939Message::from_frame(0x0, frame))
    }
}
