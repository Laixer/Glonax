use std::collections::HashMap;

use j1939::{Frame, FrameBuilder, IdBuilder, PGN};

use crate::net::Parsable;

const STATUS_PGN: u32 = 65_288;
const BANK_PGN_LIST: [PGN; 2] = [PGN::Other(40_960), PGN::Other(41_216)];
const BANK_SLOTS: usize = 4;

pub struct ActuatorMessage {
    /// Node ID
    node: u8,
    /// Actuator values
    pub actuators: [Option<i16>; 8],
}

impl ActuatorMessage {
    pub fn from_frame(node: u8, frame: &Frame) -> Self {
        let mut actuators: [Option<i16>; 8] = [None; 8];

        if frame.id().pgn() == BANK_PGN_LIST[0] {
            if frame.pdu()[0..2] != [0xff, 0xff] {
                actuators[0] = Some(i16::from_le_bytes(frame.pdu()[0..2].try_into().unwrap()));
            }
            if frame.pdu()[2..4] != [0xff, 0xff] {
                actuators[1] = Some(i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap()));
            }
            if frame.pdu()[4..6] != [0xff, 0xff] {
                actuators[2] = Some(i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap()));
            }
            if frame.pdu()[6..8] != [0xff, 0xff] {
                actuators[3] = Some(i16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap()));
            }
        } else if frame.id().pgn() == BANK_PGN_LIST[1] {
            if frame.pdu()[0..2] != [0xff, 0xff] {
                actuators[4] = Some(i16::from_le_bytes(frame.pdu()[0..2].try_into().unwrap()));
            }
            if frame.pdu()[2..4] != [0xff, 0xff] {
                actuators[5] = Some(i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap()));
            }
            if frame.pdu()[4..6] != [0xff, 0xff] {
                actuators[6] = Some(i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap()));
            }
            if frame.pdu()[6..8] != [0xff, 0xff] {
                actuators[7] = Some(i16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap()));
            }
        }

        Self { node, actuators }
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
                    .da(self.node)
                    .sa(crate::consts::DEFAULT_J1939_ADDRESS)
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
    /// Node ID
    node: u8,
    /// Motion lock
    pub locked: Option<bool>,
    /// Motion reset
    pub reset: Option<bool>,
}

impl MotionConfigMessage {
    /// Construct a locked new motion config message.
    pub fn locked(node: u8) -> Self {
        Self {
            node,
            locked: Some(true),
            reset: None,
        }
    }

    /// Construct a unlocked new motion config message.
    pub fn unlocked(node: u8) -> Self {
        Self {
            node,
            locked: Some(false),
            reset: None,
        }
    }

    pub fn reset(node: u8) -> Self {
        Self {
            node,
            locked: None,
            reset: Some(true),
        }
    }

    // TODO: Check if bytes are 0xff
    fn from_frame(node: u8, frame: &Frame) -> Self {
        Self {
            node,
            locked: Some(frame.pdu()[3] == 0x0),
            reset: Some(frame.pdu()[4] == 0x1),
        }
    }

    fn to_frame(&self) -> Vec<Frame> {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage3)
                .priority(3)
                .da(self.node)
                .sa(crate::consts::DEFAULT_J1939_ADDRESS)
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
            s.push_str(&format!("Reset: {};", if reset { "Yes" } else { "No" }));
        }

        write!(f, "{}", s)
    }
}

struct ConfigMessage {
    /// Node ID
    node: u8,
    /// Identification LED
    pub led_on: Option<bool>,
    /// Reset hardware
    pub reset: Option<bool>,
}

impl ConfigMessage {
    fn from_frame(node: u8, frame: &Frame) -> Self {
        let mut led_on = None;
        let mut reset = None;

        if frame.pdu()[2] == 0x0 {
            led_on = Some(false);
        } else if frame.pdu()[2] == 0x1 {
            led_on = Some(true);
        }
        if frame.pdu()[3] == 0x0 {
            reset = Some(false);
        } else if frame.pdu()[3] == 0x69 {
            reset = Some(true);
        }

        Self {
            node,
            led_on,
            reset,
        }
    }

    fn to_frame(&self) -> Vec<Frame> {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage1)
                .da(self.node)
                .sa(crate::consts::DEFAULT_J1939_ADDRESS)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', 0xff, 0xff]);

        if let Some(led_on) = self.led_on {
            frame_builder.as_mut()[2] = u8::from(led_on);
        }

        if let Some(reset) = self.reset {
            frame_builder.as_mut()[3] = if reset { 0x69 } else { 0x0 };
        }

        vec![frame_builder.build()]
    }
}

impl std::fmt::Display for ConfigMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config {} {}",
            if self.led_on.unwrap_or(false) {
                "LED on"
            } else {
                "LED off"
            },
            if self.reset.unwrap_or(false) {
                "reset"
            } else {
                "no reset"
            }
        )
    }
}

pub struct StatusMessage {
    /// Node ID
    node: u8,
    /// ECU status
    pub state: u8,
    /// Motion lock
    pub locked: bool,
    /// Uptime
    pub uptime: u32,
}

impl StatusMessage {
    fn from_frame(node: u8, frame: &Frame) -> Self {
        Self {
            node,
            state: frame.pdu()[0],
            locked: frame.pdu()[2] == 0x1,
            uptime: u32::from_le_bytes(frame.pdu()[4..8].try_into().unwrap()),
        }
    }
}

impl std::fmt::Display for StatusMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Status: {:#x} Motion: {} Uptime: {}",
            self.state,
            if self.locked { "Locked" } else { "Unlocked" },
            self.uptime
        )
    }
}

pub struct HydraulicControlUnit {
    /// Source and destination node ID.
    pub node: u8,
}

impl HydraulicControlUnit {
    /// Construct a new actuator service.
    pub fn new(node: u8) -> Self {
        Self { node }
    }

    /// Locks the motion controller
    pub fn lock(&self) -> Vec<Frame> {
        let msg = MotionConfigMessage::locked(self.node);

        trace!("HCU: {}", msg);

        msg.to_frame()
    }

    /// Unlocks the motion controller
    pub fn unlock(&self) -> Vec<Frame> {
        let msg = MotionConfigMessage::unlocked(self.node);

        trace!("HCU: {}", msg);

        msg.to_frame()
    }

    /// Motion reset
    pub fn motion_reset(&self) -> Vec<Frame> {
        let msg = MotionConfigMessage::reset(self.node);

        trace!("HCU: {}", msg);

        msg.to_frame()
    }

    /// Sets the LED on the motion controller
    pub fn set_led(&self, on: bool) -> Vec<Frame> {
        let msg = ConfigMessage {
            node: self.node,
            led_on: Some(on),
            reset: None,
        };

        trace!("HCU: {}", msg);

        msg.to_frame()
    }

    /// System reset
    pub fn reset(&self) -> Vec<Frame> {
        let msg = ConfigMessage {
            node: self.node,
            led_on: None,
            reset: Some(true),
        };

        trace!("HCU: {}", msg);

        msg.to_frame()
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
            node: self.node,
            actuators,
        };

        trace!("HCU: {}", msg);

        msg.to_frame()
    }
}

impl
    Parsable<(
        Option<ActuatorMessage>,
        Option<MotionConfigMessage>,
        Option<StatusMessage>,
    )> for HydraulicControlUnit
{
    fn parse(
        &mut self,
        frame: &Frame,
    ) -> Option<(
        Option<ActuatorMessage>,
        Option<MotionConfigMessage>,
        Option<StatusMessage>,
    )> {
        if frame.len() < 4 {
            return None;
        }

        if frame.id().pgn() == PGN::ProprietarilyConfigurableMessage3 {
            if frame.pdu()[0..2] != [b'Z', b'C'] {
                return None;
            }
            if frame.pdu()[2] != 0xff {
                return None;
            }

            return Some((
                None,
                Some(MotionConfigMessage::from_frame(self.node, frame)),
                None,
            ));
        }

        if frame.id().pgn() == PGN::ProprietarilyConfigurableMessage1 {
            if frame.pdu()[0..2] != [b'Z', b'C'] {
                return None;
            }

            let _config_message = ConfigMessage::from_frame(self.node, frame);
        }

        if frame.id().pgn() == PGN::ProprietaryB(STATUS_PGN) {
            let status_message = StatusMessage::from_frame(self.node, frame);

            return Some((None, None, Some(status_message)));
        }

        if frame.id().pgn() == BANK_PGN_LIST[0] || frame.id().pgn() == BANK_PGN_LIST[1] {
            if frame.len() < 8 {
                return None;
            }

            return Some((
                Some(ActuatorMessage::from_frame(self.node, frame)),
                None,
                None,
            ));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actuator_message_1() {
        let message_a = ActuatorMessage {
            node: 0x3D,
            actuators: [None; 8],
        };

        let frames = message_a.to_frame();

        assert_eq!(frames.len(), 0);
    }

    #[test]
    fn actuator_message_2() {
        let message_a = ActuatorMessage {
            node: 0x3D,
            actuators: [Some(-24_000), None, None, Some(500), None, None, None, None],
        };

        let frames = message_a.to_frame();
        let message_b = ActuatorMessage::from_frame(0x3D, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(
            message_b.actuators,
            [Some(-24_000), None, None, Some(500), None, None, None, None]
        );
    }

    #[test]
    fn actuator_message_3() {
        let message_a = ActuatorMessage {
            node: 0x3D,
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
        let message_b = ActuatorMessage::from_frame(0x3D, &frames[0]);

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
            node: 0x3D,
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
        let message_b = ActuatorMessage::from_frame(0x3D, &frames[0]);
        let message_c = ActuatorMessage::from_frame(0x3D, &frames[1]);

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
        let config_a = MotionConfigMessage::locked(0x5E);

        let frames = config_a.to_frame();
        let config_b = MotionConfigMessage::from_frame(0x5E, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert!(config_b.locked.unwrap());
    }

    #[test]
    fn motion_config_message_2() {
        let config_a = MotionConfigMessage::unlocked(0xA9);

        let frames = config_a.to_frame();
        let config_b = MotionConfigMessage::from_frame(0xA9, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert!(!config_b.locked.unwrap());
    }

    #[test]
    fn config_message_1() {
        let config_a = ConfigMessage {
            node: 0x2B,
            led_on: Some(true),
            reset: None,
        };

        let frames = config_a.to_frame();
        let config_b = ConfigMessage::from_frame(0x2B, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.led_on, Some(true));
        assert_eq!(config_b.reset, None);
    }

    #[test]
    fn config_message_2() {
        let config_a = ConfigMessage {
            node: 0x3C,
            led_on: Some(false),
            reset: None,
        };

        let frames = config_a.to_frame();
        let config_b = ConfigMessage::from_frame(0x3C, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.led_on, Some(false));
        assert_eq!(config_b.reset, None);
    }

    #[test]
    fn config_message_3() {
        let config_a = ConfigMessage {
            node: 0x4D,
            led_on: None,
            reset: Some(true),
        };

        let frames = config_a.to_frame();
        let config_b = ConfigMessage::from_frame(0x4D, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.led_on, None);
        assert_eq!(config_b.reset, Some(true));
    }
}