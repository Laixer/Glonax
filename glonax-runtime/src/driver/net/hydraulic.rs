use std::collections::HashMap;

use j1939::{Frame, FrameBuilder, IdBuilder, PDU_NOT_AVAILABLE, PGN};

use crate::net::Parsable;

use super::vecraft::{VecraftConfigMessage, VecraftStatusMessage};

const STATUS_PGN: u32 = 65_288;
const BANK_PGN_LIST: [PGN; 2] = [PGN::Other(40_960), PGN::Other(41_216)];
const BANK_SLOTS: usize = 4;

pub enum HydraulicMessage {
    Actuator(ActuatorMessage),
    MotionConfig(MotionConfigMessage),
    VecraftConfig(VecraftConfigMessage),
    Status(VecraftStatusMessage),
}

pub struct ActuatorMessage {
    /// Destination address
    destination_address: u8,
    /// Source address
    source_address: u8,
    /// Actuator values
    pub actuators: [Option<i16>; 8],
}

impl ActuatorMessage {
    pub fn from_frame(destination_address: u8, source_address: u8, frame: &Frame) -> Self {
        let mut actuators: [Option<i16>; 8] = [None; 8];

        if frame.id().pgn() == BANK_PGN_LIST[0] {
            if frame.pdu()[0..2] != [PDU_NOT_AVAILABLE, PDU_NOT_AVAILABLE] {
                actuators[0] = Some(i16::from_le_bytes(frame.pdu()[0..2].try_into().unwrap()));
            }
            if frame.pdu()[2..4] != [PDU_NOT_AVAILABLE, PDU_NOT_AVAILABLE] {
                actuators[1] = Some(i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap()));
            }
            if frame.pdu()[4..6] != [PDU_NOT_AVAILABLE, PDU_NOT_AVAILABLE] {
                actuators[2] = Some(i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap()));
            }
            if frame.pdu()[6..8] != [PDU_NOT_AVAILABLE, PDU_NOT_AVAILABLE] {
                actuators[3] = Some(i16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap()));
            }
        } else if frame.id().pgn() == BANK_PGN_LIST[1] {
            if frame.pdu()[0..2] != [PDU_NOT_AVAILABLE, PDU_NOT_AVAILABLE] {
                actuators[4] = Some(i16::from_le_bytes(frame.pdu()[0..2].try_into().unwrap()));
            }
            if frame.pdu()[2..4] != [PDU_NOT_AVAILABLE, PDU_NOT_AVAILABLE] {
                actuators[5] = Some(i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap()));
            }
            if frame.pdu()[4..6] != [PDU_NOT_AVAILABLE, PDU_NOT_AVAILABLE] {
                actuators[6] = Some(i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap()));
            }
            if frame.pdu()[6..8] != [PDU_NOT_AVAILABLE, PDU_NOT_AVAILABLE] {
                actuators[7] = Some(i16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap()));
            }
        }

        Self {
            destination_address,
            source_address,
            actuators,
        }
    }

    fn to_frame(&self) -> Vec<Frame> {
        let mut frames = vec![];

        for (idx, bank) in BANK_PGN_LIST.into_iter().enumerate() {
            let stride = idx * BANK_SLOTS;

            if !self.actuators[stride..stride + BANK_SLOTS]
                .iter()
                .any(|f| f.is_some())
            {
                continue;
            }

            let pdu: [u8; 8] = self.actuators[stride..stride + BANK_SLOTS]
                .iter()
                .flat_map(|p| p.map_or([0xff, 0xff], |v| v.to_le_bytes()))
                .collect::<Vec<u8>>()
                .as_slice()[..8]
                .try_into()
                .unwrap();

            let frame = Frame::new(
                IdBuilder::from_pgn(bank)
                    .priority(3)
                    .da(self.destination_address)
                    .sa(self.source_address)
                    .build(),
                pdu,
            );

            frames.push(frame);
        }

        frames
    }
}

impl std::fmt::Display for ActuatorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Actuator state {}",
            self.actuators
                .iter()
                .enumerate()
                .map(|(idx, act)| {
                    format!(
                        "{}: {}",
                        idx,
                        act.map_or("NaN".to_owned(), |f| f.to_string())
                    )
                })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub struct MotionConfigMessage {
    /// Destination address
    destination_address: u8,
    /// Source address
    source_address: u8,
    /// Motion lock
    pub locked: Option<bool>,
    /// Motion reset
    pub reset: Option<bool>,
}

impl MotionConfigMessage {
    fn from_frame(destination_address: u8, source_address: u8, frame: &Frame) -> Self {
        Self {
            destination_address,
            source_address,
            locked: if frame.pdu()[3] != PDU_NOT_AVAILABLE {
                Some(frame.pdu()[3] == 0x0)
            } else {
                None
            },
            reset: if frame.pdu()[4] != PDU_NOT_AVAILABLE {
                Some(frame.pdu()[4] == 0x1)
            } else {
                None
            },
        }
    }

    fn to_frame(&self) -> Vec<Frame> {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage3)
                .priority(3)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        )
        .copy_from_slice(&[
            b'Z',
            b'C',
            0xff,
            if let Some(locked) = self.locked {
                if locked {
                    0x0
                } else {
                    0x1
                }
            } else {
                0xff
            },
            if let Some(reset) = self.reset {
                if reset {
                    0x1
                } else {
                    0x0
                }
            } else {
                0xff
            },
        ])
        .build();

        vec![frame]
    }
}

impl std::fmt::Display for MotionConfigMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        if let Some(locked) = self.locked {
            s.push_str(&format!(
                "Motion: {};",
                if locked { "Locked" } else { "Unlocked" }
            ));
        }

        if let Some(reset) = self.reset {
            s.push_str(&format!(
                "Motion reset: {};",
                if reset { "Yes" } else { "No" }
            ));
        }

        write!(f, "{}", s)
    }
}

pub struct HydraulicControlUnit {
    /// Destination address.
    pub destination_address: u8,
    /// Source address.
    pub source_address: u8,
}

impl HydraulicControlUnit {
    /// Construct a new actuator service.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
        }
    }

    /// Locks the motion controller
    pub fn lock(&self) -> Vec<Frame> {
        MotionConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            locked: Some(true),
            reset: None,
        }
        .to_frame()
    }

    /// Unlocks the motion controller
    pub fn unlock(&self) -> Vec<Frame> {
        MotionConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            locked: Some(false),
            reset: None,
        }
        .to_frame()
    }

    /// Motion reset
    pub fn motion_reset(&self) -> Vec<Frame> {
        MotionConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            locked: None,
            reset: Some(true),
        }
        .to_frame()
    }

    /// Set or unset identification mode.
    pub fn set_ident(&self, on: bool) -> Vec<Frame> {
        VecraftConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: Some(on),
            reboot: false,
        }
        .to_frame()
    }

    /// System reboot / reset
    pub fn reboot(&self) -> Vec<Frame> {
        VecraftConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: None,
            reboot: true,
        }
        .to_frame()
    }

    // FUTURE: Move this to HCU
    /// Drive both tracks
    pub fn drive_straight(&self, value: i16) -> Vec<Frame> {
        self.actuator_command([(2, value), (3, value)].into_iter().collect())
    }

    /// Sends a command to the motion controller
    pub fn actuator_command(&self, actuator_command: HashMap<u8, i16>) -> Vec<Frame> {
        let mut actuators = [None; 8];

        for (actuator, value) in actuator_command {
            actuators[actuator as usize] = Some(value);
        }

        let msg = ActuatorMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            actuators,
        };

        trace!("HCU: {}", msg);

        msg.to_frame()
    }
}

impl Parsable<HydraulicMessage> for HydraulicControlUnit {
    fn parse(&mut self, frame: &Frame) -> Option<HydraulicMessage> {
        if frame.id().sa() != self.destination_address {
            return None;
        }

        match frame.id().pgn() {
            PGN::ProprietarilyConfigurableMessage3 => {
                if frame.pdu()[0..2] != [b'Z', b'C'] {
                    return None;
                }
                if frame.pdu()[2] != 0xff {
                    return None;
                }

                Some(HydraulicMessage::MotionConfig(
                    MotionConfigMessage::from_frame(
                        self.destination_address,
                        self.source_address,
                        frame,
                    ),
                ))
            }
            PGN::ProprietarilyConfigurableMessage1 => {
                if frame.pdu()[0..2] != [b'Z', b'C'] {
                    return None;
                }

                Some(HydraulicMessage::VecraftConfig(
                    VecraftConfigMessage::from_frame(
                        self.destination_address,
                        self.source_address,
                        frame,
                    ),
                ))
            }
            PGN::ProprietaryB(STATUS_PGN) => {
                Some(HydraulicMessage::Status(VecraftStatusMessage::from_frame(
                    self.destination_address,
                    self.source_address,
                    frame,
                )))
            }
            PGN::Other(40_960) | PGN::Other(41_216) => {
                if frame.len() < 8 {
                    return None;
                }

                Some(HydraulicMessage::Actuator(ActuatorMessage::from_frame(
                    self.destination_address,
                    self.source_address,
                    frame,
                )))
            }
            _ => None,
        }
    }
}

impl super::J1939Unit for HydraulicControlUnit {
    fn try_accept(
        &mut self,
        router: &mut crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) {
        if let Some(message) = router.try_accept(self) {
            if let Ok(_runtime_state) = runtime_state.try_write() {
                match message {
                    HydraulicMessage::Actuator(_actuator) => {
                        // runtime_state.state.actuators.insert(
                        //     (self.destination_address, self.source_address),
                        //     actuator.actuators,
                        // );
                    }
                    HydraulicMessage::MotionConfig(_config) => {
                        // runtime_state
                        //     .state
                        //     .motion_config
                        //     .insert((self.destination_address, self.source_address), config);
                    }
                    HydraulicMessage::VecraftConfig(_config) => {
                        // runtime_state
                        //     .state
                        //     .vecraft_config
                        //     .insert((self.destination_address, self.source_address), config);
                    }
                    HydraulicMessage::Status(_status) => {
                        // runtime_state
                        //     .state
                        //     .hcu_status
                        //     .insert((self.destination_address, self.source_address), status);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actuator_message_1() {
        let message_a = ActuatorMessage {
            destination_address: 0x3D,
            source_address: 0x8B,
            actuators: [None; 8],
        };

        let frames = message_a.to_frame();

        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn actuator_message_2() {
        let message_a = ActuatorMessage {
            destination_address: 0x3D,
            source_address: 0x8B,
            actuators: [Some(-24_000), None, None, Some(500), None, None, None, None],
        };

        let frames = message_a.to_frame();
        let message_b = ActuatorMessage::from_frame(0x3D, 0x8B, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(
            message_b.actuators,
            [Some(-24_000), None, None, Some(500), None, None, None, None]
        );
    }

    #[test]
    fn actuator_message_3() {
        let message_a = ActuatorMessage {
            destination_address: 0x3D,
            source_address: 0x8B,
            actuators: [
                None,
                None,
                None,
                None,
                Some(32_000),
                Some(i16::MAX),
                None,
                None,
            ],
        };

        let frames = message_a.to_frame();
        let message_b = ActuatorMessage::from_frame(0x3D, 0x8B, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(
            message_b.actuators,
            [
                None,
                None,
                None,
                None,
                Some(32_000),
                Some(i16::MAX),
                None,
                None
            ]
        );
    }

    #[test]
    fn actuator_message_4() {
        let message_a = ActuatorMessage {
            destination_address: 0x3D,
            source_address: 0x8B,
            actuators: [
                Some(-100),
                Some(200),
                Some(-300),
                Some(400),
                Some(-500),
                Some(600),
                Some(-700),
                Some(800),
            ],
        };

        let frames = message_a.to_frame();
        let message_b = ActuatorMessage::from_frame(0x3D, 0x8B, &frames[0]);
        let message_c = ActuatorMessage::from_frame(0x3D, 0x8B, &frames[1]);

        assert_eq!(frames.len(), 2);

        assert_eq!(
            message_b.actuators,
            [
                Some(-100),
                Some(200),
                Some(-300),
                Some(400),
                None,
                None,
                None,
                None
            ]
        );
        assert_eq!(
            message_c.actuators,
            [
                None,
                None,
                None,
                None,
                Some(-500),
                Some(600),
                Some(-700),
                Some(800)
            ]
        );
    }

    #[test]
    fn motion_config_message_1() {
        let frames = MotionConfigMessage {
            destination_address: 0x5E,
            source_address: 0xEE,
            locked: Some(true),
            reset: None,
        }
        .to_frame();

        let config_b = MotionConfigMessage::from_frame(0x5E, 0xEE, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.locked, Some(true));
        assert_eq!(config_b.reset, None)
    }

    #[test]
    fn motion_config_message_2() {
        let frames = MotionConfigMessage {
            destination_address: 0xA9,
            source_address: 0x11,
            locked: Some(false),
            reset: None,
        }
        .to_frame();

        let config_b = MotionConfigMessage::from_frame(0xA9, 0x11, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.locked, Some(false));
        assert_eq!(config_b.reset, None)
    }

    #[test]
    fn motion_config_message_3() {
        let frames = MotionConfigMessage {
            destination_address: 0x66,
            source_address: 0x22,
            locked: None,
            reset: Some(true),
        }
        .to_frame();

        let config_b = MotionConfigMessage::from_frame(0x66, 0x22, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.locked, None);
        assert_eq!(config_b.reset, Some(true));
    }

    #[test]
    fn motion_config_message_4() {
        let frames = MotionConfigMessage {
            destination_address: 0x66,
            source_address: 0x22,
            locked: None,
            reset: Some(false),
        }
        .to_frame();

        let config_b = MotionConfigMessage::from_frame(0x66, 0x22, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.locked, None);
        assert_eq!(config_b.reset, Some(false));
    }

    #[test]
    fn config_message_1() {
        let config_a = VecraftConfigMessage {
            destination_address: 0x2B,
            source_address: 0x4D,
            ident_on: Some(true),
            reboot: false,
        };

        let frames = config_a.to_frame();
        let config_b = VecraftConfigMessage::from_frame(0x2B, 0x4D, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.ident_on, Some(true));
        assert!(!config_b.reboot);
    }

    #[test]
    fn config_message_2() {
        let config_a = VecraftConfigMessage {
            destination_address: 0x3C,
            source_address: 0x4F,
            ident_on: Some(false),
            reboot: false,
        };

        let frames = config_a.to_frame();
        let config_b = VecraftConfigMessage::from_frame(0x3C, 0x4F, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.ident_on, Some(false));
        assert!(!config_b.reboot);
    }

    #[test]
    fn config_message_3() {
        let config_a = VecraftConfigMessage {
            destination_address: 0x4D,
            source_address: 0xCD,
            ident_on: None,
            reboot: true,
        };

        let frames = config_a.to_frame();
        let config_b = VecraftConfigMessage::from_frame(0x4D, 0xCD, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.ident_on, None);
        assert!(config_b.reboot);
    }
}
