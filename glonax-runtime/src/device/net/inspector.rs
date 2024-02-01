use j1939::{decode, Frame, PGN};

use crate::net::Parsable;

#[derive(Default)]
pub struct J1939ApplicationInspector;

#[derive(Debug)]
pub struct J1939Name {
    /// Identity number.
    pub identity_number: u32,
    /// Manufacturer code.
    pub manufacturer_code: u16,
    /// Function instance.
    pub function_instance: u8,
    /// ECU instance.
    pub ecu_instance: u8,
    /// Function.
    pub function: u8,
    /// Vehicle system.
    pub vehicle_system: u8,
    /// Vehicle system instance.
    pub vehicle_system_instance: u8,
    /// Industry group.
    pub industry_group: u8,
    /// Arbitrary address.
    pub arbitrary_address: u8,
}

impl std::fmt::Display for J1939Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Identity number: 0x{:X}; Manufacturer code: 0x{:X}; Function instance: 0x{:X}; ECU instance: 0x{:X}; Function: 0x{:X}; Vehicle system: 0x{:X}; Vehicle system instance: 0x{:X}; Industry group: {:X}; Arbitrary address: {}",
            self.identity_number,
            self.manufacturer_code,
            self.function_instance,
            self.ecu_instance,
            self.function,
            self.vehicle_system,
            self.vehicle_system_instance,
            self.industry_group,
            self.arbitrary_address == 1
        )
    }
}

pub enum J1939Message {
    /// Software identification.
    SoftwareIndent((u8, u8, u8)),
    /// Requested PGN.
    RequestPGN(u32),
    /// Address claim.
    AddressClaim(J1939Name),
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
            PGN::AddressClaimed => {
                // [18EEFF4A] Prio: 6 PGN: 60928 DA: 0xFF    [09, 03, 4B, 24, 11, 05, 06, 05]
                // [18EEFF6B] Prio: 6 PGN: 60928 DA: 0xFF    [19, A4, 49, 24, 11, 05, 06, 85]

                // 0b_0001_1001, // 0x19 Identity number
                // 0b_1010_0100, // 0xA4 Identity number
                //
                // 0b_0100_1001, // 0x49 Manufacturer code
                // 0b_0010_0100, // 0x24 Manufacturer code
                //
                // 0b_0001_0001, // 0x11 Function Instance | ECU Instance
                // 0x05,         // 0x05 Function
                // 0b_0000_0110, // 0x06 Vehicle System
                // 0b_1000_0101, // 0x85 Arbitrary Address Capable | Industry Group | Vehicle System Instance

                let identity_number = frame.pdu()[0] as u32
                    | ((frame.pdu()[1] as u32) << 8)
                    | (((frame.pdu()[2] & 0b01001) as u32) << 16);

                let manufacturer_code =
                    (frame.pdu()[2] >> 5) as u16 | ((frame.pdu()[3] as u16) << 3);

                let function_instance = frame.pdu()[4] >> 3;
                let ecu_instance = frame.pdu()[4] & 0b111;

                let function = frame.pdu()[5];

                let vehicle_system = frame.pdu()[6] & 0b0111_1111;

                let vehicle_system_instance = frame.pdu()[7] & 0b1111;
                let industry_group = frame.pdu()[7] & 0b0111_0000;
                let arbitrary_address = frame.pdu()[7] >> 7;

                Some(Self::AddressClaim(J1939Name {
                    identity_number,
                    manufacturer_code,
                    function_instance,
                    ecu_instance,
                    function,
                    vehicle_system,
                    vehicle_system_instance,
                    industry_group,
                    arbitrary_address,
                }))
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
