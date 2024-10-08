use std::collections::HashMap;

use j1939::{protocol, Frame, FrameBuilder, IdBuilder, Name, PDU_NOT_AVAILABLE, PGN};

use crate::{
    core::{Motion, Object, ObjectMessage},
    net::Parsable,
    runtime::{J1939Unit, J1939UnitError, NetDriverContext},
};

use super::vecraft::{VecraftConfigMessage, VecraftFactoryResetMessage, VecraftStatusMessage};

const STATUS_PGN: u32 = 65_288;
const BANK_PGN_LIST: [PGN; 2] = [PGN::Other(40_960), PGN::Other(41_216)];
const BANK_SLOTS: usize = 4;

pub enum HydraulicMessage {
    Actuator(ActuatorMessage),
    MotionConfig(MotionConfigMessage),
    VecraftConfig(VecraftConfigMessage),
    SoftwareIdentification((u8, u8, u8)),
    AddressClaim(Name),
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
            "State {}",
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

    fn to_frame(&self) -> Frame {
        FrameBuilder::new(
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
        .build()
    }
}

impl std::fmt::Display for MotionConfigMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Locked: {} Reset: {}",
            if self.locked == Some(true) {
                "Yes"
            } else {
                "No"
            },
            if self.reset == Some(true) {
                "Yes"
            } else {
                "No"
            }
        )
    }
}

#[derive(Clone)]
pub struct HydraulicControlUnit {
    /// Network interface.
    interface: String,
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
}

impl HydraulicControlUnit {
    /// Construct a new actuator service.
    pub fn new(interface: &str, da: u8, sa: u8) -> Self {
        Self {
            interface: interface.to_string(),
            destination_address: da,
            source_address: sa,
        }
    }

    /// Locks the motion controller
    pub fn lock(&self) -> Frame {
        MotionConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            locked: Some(true),
            reset: None,
        }
        .to_frame()
    }

    /// Unlocks the motion controller
    pub fn unlock(&self) -> Frame {
        MotionConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            locked: Some(false),
            reset: None,
        }
        .to_frame()
    }

    /// Motion reset
    pub fn motion_reset(&self) -> Frame {
        MotionConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            locked: None,
            reset: Some(true),
        }
        .to_frame()
    }

    /// Set or unset identification mode.
    pub fn set_ident(&self, on: bool) -> Frame {
        VecraftConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: Some(on),
            reboot: false,
        }
        .to_frame()
    }

    /// System reboot
    pub fn reboot(&self) -> Frame {
        VecraftConfigMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            ident_on: None,
            reboot: true,
        }
        .to_frame()
    }

    /// Factory reset
    pub fn factory_reset(&self) -> Frame {
        VecraftFactoryResetMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
        }
        .to_frame()
    }

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

        let message = ActuatorMessage {
            destination_address: self.destination_address,
            source_address: self.source_address,
            actuators,
        };

        message.to_frame()
    }
}

impl Parsable<HydraulicMessage> for HydraulicControlUnit {
    fn parse(&self, frame: &Frame) -> Option<HydraulicMessage> {
        if let Some(destination_address) = frame.id().destination_address() {
            if destination_address != self.destination_address && destination_address != 0xff {
                return None;
            }
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
            PGN::SoftwareIdentification => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

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

                        Some(HydraulicMessage::SoftwareIdentification((
                            major, minor, patch,
                        )))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            PGN::AddressClaimed => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(HydraulicMessage::AddressClaim(Name::from_bytes(
                    frame.pdu().try_into().unwrap(),
                )))
            }
            PGN::ProprietaryB(STATUS_PGN) => {
                if frame.id().source_address() != self.destination_address {
                    return None;
                }

                Some(HydraulicMessage::Status(VecraftStatusMessage::from_frame(
                    frame,
                )))
            }
            PGN::Other(40_960) | PGN::Other(41_216) => Some(HydraulicMessage::Actuator(
                ActuatorMessage::from_frame(self.destination_address, self.source_address, frame),
            )),
            _ => None,
        }
    }
}

impl J1939Unit for HydraulicControlUnit {
    fn vendor(&self) -> &'static str {
        "laixer"
    }

    fn product(&self) -> &'static str {
        "hcu"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    fn setup(
        &self,
        _ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::AddressClaimed,
        ));
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::SoftwareIdentification,
        ));
        tx_queue.push(protocol::request(
            self.destination_address,
            self.source_address,
            PGN::ComponentIdentification,
        ));

        tx_queue.push(self.motion_reset());
        tx_queue.push(self.set_ident(true));
        tx_queue.push(self.set_ident(false));

        Ok(())
    }

    fn teardown(
        &self,
        _ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        tx_queue.push(self.motion_reset());

        Ok(())
    }

    fn try_recv(
        &self,
        ctx: &mut NetDriverContext,
        frame: &j1939::Frame,
        rx_queue: &mut Vec<Object>,
    ) -> Result<(), J1939UnitError> {
        if let Some(message) = self.parse(frame) {
            match message {
                HydraulicMessage::Actuator(_actuator) => {}
                HydraulicMessage::MotionConfig(_config) => {}
                HydraulicMessage::VecraftConfig(_config) => {}
                HydraulicMessage::SoftwareIdentification(version) => {
                    debug!(
                        "[{}] {}: Firmware version: {}.{}.{}",
                        self.interface,
                        self.name(),
                        version.0,
                        version.1,
                        version.2
                    );

                    ctx.rx_mark();

                    return Ok(());
                }
                HydraulicMessage::AddressClaim(name) => {
                    debug!(
                        "[{}] {}: Address claimed: {}",
                        self.interface,
                        self.name(),
                        name
                    );

                    ctx.rx_mark();

                    return Ok(());
                }
                HydraulicMessage::Status(status) => {
                    // let object =
                    //     Object::Control(crate::core::Control::HydraulicLock(status.locked));

                    // ctx.set_rx_last_message(ObjectMessage::signal(object.clone()));

                    // rx_queue.push(object);

                    let motion = if status.locked {
                        Motion::StopAll
                    } else {
                        Motion::ResumeAll
                    };

                    ctx.set_rx_last_message(ObjectMessage::signal(Object::Motion(motion.clone())));

                    rx_queue.push(Object::Motion(motion.clone()));

                    status.into_error()?;

                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn trigger(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
        object: &Object,
    ) -> Result<(), J1939UnitError> {
        // if let Object::Control(crate::core::Control::HydraulicLock(lock)) = object {
        //     trace!(
        //         "[{}] {}: Hydraulic lock: {}",
        //         self.interface,
        //         self.name(),
        //         lock
        //     );

        //     ctx.set_tx_last_message(ObjectMessage::command(object.clone()));

        //     if *lock {
        //         tx_queue.push(self.lock());
        //     } else {
        //         tx_queue.push(self.unlock());
        //     }
        // }

        if let Object::Motion(motion) = object {
            trace!(
                "[{}] {}: Hydraulic: {}",
                self.interface,
                self.name(),
                motion
            );

            ctx.set_tx_last_message(ObjectMessage::command(object.clone()));

            // TODO: StopAll, ResumeAll, ResetAll should be moved into Control::HydraulicXXX
            match motion {
                Motion::StopAll => {
                    tx_queue.push(self.lock());
                }
                Motion::ResumeAll => {
                    tx_queue.push(self.unlock());
                }
                Motion::ResetAll => {
                    tx_queue.push(self.motion_reset());
                }
                Motion::StraightDrive(value) => {
                    let frames = &self.drive_straight(*value);
                    tx_queue.extend_from_slice(frames);
                }
                Motion::Change(changes) => {
                    let frames = &self.actuator_command(
                        changes
                            .iter()
                            .map(|changeset| (changeset.actuator as u8, changeset.value))
                            .collect(),
                    );

                    tx_queue.extend_from_slice(frames);
                }
            }
        }

        Ok(())
    }

    fn tick(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        let motion_command = {
            if let Some(message) = &ctx.tx_last_message() {
                if let Object::Motion(motion) = &message.object {
                    motion.clone()
                } else {
                    Motion::StopAll
                }
            } else {
                Motion::StopAll
            }
        };

        trace!(
            "[{}] {}: Hydraulic: {}",
            self.interface,
            self.name(),
            motion_command
        );

        match &motion_command {
            Motion::StopAll => {
                tx_queue.push(self.lock());
            }
            Motion::ResumeAll => {
                tx_queue.push(self.unlock());
            }
            Motion::ResetAll => {
                tx_queue.push(self.motion_reset());
            }
            Motion::StraightDrive(value) => {
                let frames = &self.drive_straight(*value);
                tx_queue.extend_from_slice(frames);
            }
            Motion::Change(changes) => {
                let frames = &self.actuator_command(
                    changes
                        .iter()
                        .map(|changeset| (changeset.actuator as u8, changeset.value))
                        .collect(),
                );

                tx_queue.extend_from_slice(frames);
            }
        }

        Ok(())
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
        let frame = MotionConfigMessage {
            destination_address: 0x5E,
            source_address: 0xEE,
            locked: Some(true),
            reset: None,
        }
        .to_frame();

        let config_b = MotionConfigMessage::from_frame(0x5E, 0xEE, &frame);

        assert_eq!(config_b.locked, Some(true));
        assert_eq!(config_b.reset, None)
    }

    #[test]
    fn motion_config_message_2() {
        let frame = MotionConfigMessage {
            destination_address: 0xA9,
            source_address: 0x11,
            locked: Some(false),
            reset: None,
        }
        .to_frame();

        let config_b = MotionConfigMessage::from_frame(0xA9, 0x11, &frame);

        assert_eq!(config_b.locked, Some(false));
        assert_eq!(config_b.reset, None)
    }

    #[test]
    fn motion_config_message_3() {
        let frame = MotionConfigMessage {
            destination_address: 0x66,
            source_address: 0x22,
            locked: None,
            reset: Some(true),
        }
        .to_frame();

        let config_b = MotionConfigMessage::from_frame(0x66, 0x22, &frame);

        assert_eq!(config_b.locked, None);
        assert_eq!(config_b.reset, Some(true));
    }

    #[test]
    fn motion_config_message_4() {
        let frame = MotionConfigMessage {
            destination_address: 0x66,
            source_address: 0x22,
            locked: None,
            reset: Some(false),
        }
        .to_frame();

        let config_b = MotionConfigMessage::from_frame(0x66, 0x22, &frame);

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

        let frame = config_a.to_frame();
        let config_b = VecraftConfigMessage::from_frame(0x2B, 0x4D, &frame);

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

        let frame = config_a.to_frame();
        let config_b = VecraftConfigMessage::from_frame(0x3C, 0x4F, &frame);

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

        let frame = config_a.to_frame();
        let config_b = VecraftConfigMessage::from_frame(0x4D, 0xCD, &frame);

        assert_eq!(config_b.ident_on, None);
        assert!(config_b.reboot);
    }
}
