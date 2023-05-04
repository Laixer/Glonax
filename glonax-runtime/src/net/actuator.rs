use std::collections::HashMap;

use glonax_j1939::*;

use super::{J1939Network, Parsable, Routable};

const BANK_PGN_LIST: [PGN; 2] = [PGN::Other(40_960), PGN::Other(41_216)];
const BANK_SLOTS: usize = 4;

pub struct ActuatorService {
    net: Option<J1939Network>,
    pub node: u8,
    pub actuators: [Option<i16>; 8],
}

impl Routable for ActuatorService {
    fn decode(&mut self, frame: &Frame) -> bool {
        if frame.len() != 8 {
            return false;
        }
        if frame.id().sa() != self.node {
            return false;
        }
        if frame.id().pgn() == PGN::Other(40_960) {
            if frame.pdu()[0..2] != [0xff, 0xff] {
                self.actuators[0] = Some(i16::from_le_bytes(frame.pdu()[0..2].try_into().unwrap()));
            }
            if frame.pdu()[2..4] != [0xff, 0xff] {
                self.actuators[1] = Some(i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap()));
            }
            if frame.pdu()[4..6] != [0xff, 0xff] {
                self.actuators[2] = Some(i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap()));
            }
            if frame.pdu()[6..8] != [0xff, 0xff] {
                self.actuators[3] = Some(i16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap()));
            }
            true
        } else if frame.id().pgn() == PGN::Other(41_216) {
            if frame.pdu()[0..2] != [0xff, 0xff] {
                self.actuators[4] = Some(i16::from_le_bytes(frame.pdu()[0..2].try_into().unwrap()));
            }
            if frame.pdu()[2..4] != [0xff, 0xff] {
                self.actuators[5] = Some(i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap()));
            }
            if frame.pdu()[4..6] != [0xff, 0xff] {
                self.actuators[6] = Some(i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap()));
            }
            if frame.pdu()[6..8] != [0xff, 0xff] {
                self.actuators[7] = Some(i16::from_le_bytes(frame.pdu()[6..8].try_into().unwrap()));
            }
            true
        } else {
            false
        }
    }

    fn encode(&self) -> Vec<Frame> {
        let mut frames = vec![];

        trace!("{}", self);

        for (idx, bank) in BANK_PGN_LIST.into_iter().enumerate() {
            let stride = idx * BANK_SLOTS;

            let pdu: [u8; 8] = self.actuators[stride..stride + BANK_SLOTS]
                .iter()
                .flat_map(|p| p.map_or([0xff, 0xff], |v| v.to_le_bytes()))
                .collect::<Vec<u8>>()
                .as_slice()[..8]
                .try_into()
                .unwrap();

            let frame = Frame::new(IdBuilder::from_pgn(bank).da(self.node).build(), pdu);

            frames.push(frame);
        }

        frames
    }
}

impl std::fmt::Display for ActuatorService {
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

pub struct ActuatorMessage {
    /// Node ID
    node: u8,
    /// Actuator values
    pub actuators: [Option<i16>; 8],
}

impl ActuatorMessage {
    pub fn from_frame(node: u8, frame: &Frame) -> Self {
        let mut actuators: [Option<i16>; 8] = [None; 8];

        if frame.id().pgn() == PGN::Other(40_960) {
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
        } else if frame.id().pgn() == PGN::Other(41_216) {
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

            let pdu: [u8; 8] = self.actuators[stride..stride + BANK_SLOTS]
                .iter()
                .flat_map(|p| p.map_or([0xff, 0xff], |v| v.to_le_bytes()))
                .collect::<Vec<u8>>()
                .as_slice()[..8]
                .try_into()
                .unwrap();

            let frame = Frame::new(IdBuilder::from_pgn(bank).da(self.node).build(), pdu);

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
    /// Lock or unlock
    pub locked: bool,
}

impl MotionConfigMessage {
    pub fn locked(node: u8) -> Self {
        trace!("Disable motion");

        Self { node, locked: true }
    }

    pub fn unlocked(node: u8) -> Self {
        trace!("Enable motion");

        Self {
            node,
            locked: false,
        }
    }

    fn from_frame(node: u8, frame: &Frame) -> Self {
        Self {
            node,
            locked: frame.pdu()[3] == 0x0,
        }
    }

    // TODO: Rename to to_frame
    fn encode(&self) -> Vec<Frame> {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage3)
                .da(self.node)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', 0xff, if self.locked { 0x0 } else { 0x1 }])
        .build();

        vec![frame]
    }
}

impl std::fmt::Display for MotionConfigMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Motion config {}",
            if self.locked { "locked" } else { "unlocked" }
        )
    }
}

struct ConfigMessage {
    /// Node ID
    node: u8,
    /// LED on or off
    pub led_on: Option<bool>,
    /// Reset or not
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

    fn encode(&self) -> Vec<Frame> {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage1)
                .da(0xff)
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

    fn decode(&mut self, frame: &Frame) -> bool {
        if frame.len() < 4 {
            return false;
        }
        if frame.id().pgn() != PGN::ProprietarilyConfigurableMessage1 {
            return false;
        }
        if frame.pdu()[0..2] != [b'Z', b'C'] {
            return false;
        }
        if frame.pdu()[2] == 0x0 {
            self.led_on = Some(false);
        } else if frame.pdu()[2] == 0x1 {
            self.led_on = Some(true);
        }
        if frame.pdu()[3] == 0x0 {
            self.reset = Some(false);
        } else if frame.pdu()[3] == 0x69 {
            self.reset = Some(true);
        }

        true
    }
}

impl ActuatorService {
    pub fn new(net: J1939Network, node: u8) -> Self {
        Self {
            net: Some(net),
            node,
            actuators: [None; 8],
        }
    }

    pub fn new2(node: u8) -> Self {
        Self {
            net: None,
            node,
            actuators: [None; 8],
        }
    }

    /// Locks the motion controller
    pub fn lock(&self) -> Vec<Frame> {
        MotionConfigMessage::locked(self.node).encode()
    }

    /// Unlocks the motion controller
    pub fn unlock(&self) -> Vec<Frame> {
        MotionConfigMessage::unlocked(self.node).encode()
    }

    /// Sets the LED on the motion controller
    pub fn set_led(&self, on: bool) -> Vec<Frame> {
        ConfigMessage {
            node: self.node,
            led_on: Some(on),
            reset: None,
        }
        .encode()
    }

    /// Resets the motion controller
    pub fn reset(&self) -> Vec<Frame> {
        ConfigMessage {
            node: self.node,
            led_on: None,
            reset: Some(true),
        }
        .encode()
    }

    // TODO: Maybe move into a trait
    pub async fn actuate(&mut self, motion: crate::transport::Motion) {
        match motion.r#type() {
            crate::transport::motion::MotionType::None => panic!("NONE should not be used"),
            crate::transport::motion::MotionType::StopAll => {
                let msg = MotionConfigMessage::locked(self.node);
                self.net
                    .as_ref()
                    .unwrap()
                    .send_vectored(&msg.encode())
                    .await
                    .unwrap();
            }
            crate::transport::motion::MotionType::ResumeAll => {
                let msg = MotionConfigMessage::unlocked(self.node);
                self.net
                    .as_ref()
                    .unwrap()
                    .send_vectored(&msg.encode())
                    .await
                    .unwrap();
            }
            crate::transport::motion::MotionType::Change => {
                self.actuator_control(
                    motion
                        .changes
                        .into_iter()
                        .map(|changeset| (changeset.actuator as u8, changeset.value as i16))
                        .collect(),
                )
                .await;
            }
        }
    }

    // TODO: If possible make immutable.
    pub async fn actuator_control(&mut self, actuators: HashMap<u8, i16>) {
        let mut bank_update = [false; 2];

        for (act, val) in &actuators {
            self.actuators[*act as usize] = Some(*val);

            bank_update[*act as usize / BANK_SLOTS] = true;
        }

        trace!("{}", self);

        for (idx, bank) in BANK_PGN_LIST.into_iter().enumerate() {
            if !bank_update[idx] {
                continue;
            }

            let stride = idx * BANK_SLOTS;

            let pdu: [u8; 8] = self.actuators[stride..stride + BANK_SLOTS]
                .iter()
                .flat_map(|p| p.map_or([0xff, 0xff], |v| v.to_le_bytes()))
                .collect::<Vec<u8>>()
                .as_slice()[..8]
                .try_into()
                .unwrap();

            let frame = Frame::new(IdBuilder::from_pgn(bank).da(self.node).build(), pdu);
            self.net.as_ref().unwrap().send(&frame).await.unwrap();
        }
    }
}

impl Parsable<(Option<ActuatorMessage>, Option<MotionConfigMessage>)> for ActuatorService {
    fn parse(
        &mut self,
        frame: &Frame,
    ) -> Option<(Option<ActuatorMessage>, Option<MotionConfigMessage>)> {
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
            ));
        }

        if frame.id().pgn() == PGN::Other(40_960) || frame.id().pgn() == PGN::Other(41_216) {
            if frame.len() < 8 {
                return None;
            }

            return Some((Some(ActuatorMessage::from_frame(self.node, frame)), None));
        }

        None
    }
}

mod tests {
    use super::*;

    #[test]
    fn motion_config_message_1() {
        let config_a = MotionConfigMessage::locked(0x5E);

        let frames = config_a.encode();
        let config_b = MotionConfigMessage::from_frame(0x5E, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.locked, true);
    }

    #[test]
    fn motion_config_message_2() {
        let config_a = MotionConfigMessage::unlocked(0xA9);

        let frames = config_a.encode();
        let config_b = MotionConfigMessage::from_frame(0xA9, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.locked, false);
    }

    #[test]
    fn config_message_1() {
        let config_a = ConfigMessage {
            node: 0x2B,
            led_on: Some(true),
            reset: None,
        };

        let frames = config_a.encode();

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

        let frames = config_a.encode();

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

        let frames = config_a.encode();

        let config_b = ConfigMessage::from_frame(0x4D, &frames[0]);

        assert_eq!(frames.len(), 1);
        assert_eq!(config_b.led_on, None);
        assert_eq!(config_b.reset, Some(true));
    }
}
