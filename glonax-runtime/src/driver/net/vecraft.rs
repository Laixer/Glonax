use j1939::{Frame, FrameBuilder, IdBuilder, PDU_NOT_AVAILABLE, PGN};

use crate::runtime::J1939UnitError;

// TODO: Remove the header field.
// TODO: Add J1939 node address
// TODO: Add CAN termination
// TODO: Add CAN bitrate
pub struct VecraftConfigMessage {
    /// Destination address
    pub(crate) destination_address: u8,
    /// Source address
    pub(crate) source_address: u8,
    /// Identification mode
    pub ident_on: Option<bool>,
    /// Hardware reboot
    pub reboot: bool,
}

impl VecraftConfigMessage {
    pub(crate) fn from_frame(destination_address: u8, source_address: u8, frame: &Frame) -> Self {
        let mut ident_on = None;
        let mut reboot = false;

        if frame.pdu()[2] != PDU_NOT_AVAILABLE {
            if frame.pdu()[2] == 0x0 {
                ident_on = Some(false);
            } else if frame.pdu()[2] == 0x1 {
                ident_on = Some(true);
            }
        }

        if frame.pdu()[3] != PDU_NOT_AVAILABLE && frame.pdu()[3] == 0x69 {
            reboot = true
        }

        Self {
            destination_address,
            source_address,
            ident_on,
            reboot,
        }
    }

    pub(crate) fn to_frame(&self) -> Frame {
        let mut frame_builder = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage1)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        )
        .copy_from_slice(&[b'Z', b'C', PDU_NOT_AVAILABLE, PDU_NOT_AVAILABLE]);

        if let Some(led_on) = self.ident_on {
            frame_builder.as_mut()[2] = u8::from(led_on);
        }

        if self.reboot {
            frame_builder.as_mut()[3] = 0x69;
        }

        frame_builder.build()
    }
}

impl std::fmt::Display for VecraftConfigMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ident: {} Reboot: {}",
            if self.ident_on == Some(true) {
                "Yes"
            } else {
                "No"
            },
            if self.reboot { "Yes" } else { "No" }
        )
    }
}

pub struct VecraftFactoryResetMessage {
    /// Destination address
    pub(crate) destination_address: u8,
    /// Source address
    pub(crate) source_address: u8,
}

impl VecraftFactoryResetMessage {
    pub(crate) fn to_frame(&self) -> Frame {
        FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage2)
                .da(self.destination_address)
                .sa(self.source_address)
                .build(),
        )
        .copy_from_slice(&[0x1; 8])
        .build()
    }
}

impl std::fmt::Display for VecraftFactoryResetMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Factory reset")
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum State {
    /// ECU is in nominal state.
    Nominal,
    /// ECU is in identification state.
    Ident,
    /// ECU is in faulty state.
    FaultyGenericError,
    /// ECU is in faulty bus state.
    FaultyBusError,
}

impl State {
    pub fn to_byte(self) -> u8 {
        match self {
            State::Nominal => 0x14,
            State::Ident => 0x16,
            State::FaultyGenericError => 0xfa,
            State::FaultyBusError => 0xfb,
        }
    }
}

impl From<u8> for State {
    fn from(byte: u8) -> Self {
        match byte {
            0x14 => State::Nominal,
            0x16 => State::Ident,
            0xfa => State::FaultyGenericError,
            0xfb => State::FaultyBusError,
            _ => panic!("Invalid state byte: {:#x}", byte),
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// TODO: Remove the lock field. Not all Vecrafts have a lock.
pub struct VecraftStatusMessage {
    /// ECU status.
    pub state: State,
    /// Motion lock.
    pub locked: bool,
    /// Uptime in seconds.
    pub uptime: u32,
}

impl VecraftStatusMessage {
    pub(crate) fn from_frame(frame: &Frame) -> Self {
        Self {
            state: State::from(frame.pdu()[0]),
            locked: frame.pdu()[2] != PDU_NOT_AVAILABLE && frame.pdu()[2] == 0x1,
            uptime: u32::from_le_bytes(frame.pdu()[4..8].try_into().unwrap()),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn to_frame(&self, destination_address: u8, source_address: u8) -> Frame {
        FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietarilyConfigurableMessage2)
                .da(destination_address)
                .sa(source_address)
                .build(),
        )
        .copy_from_slice(&[
            self.state.to_byte(),
            PDU_NOT_AVAILABLE,
            if self.locked { 0x1 } else { 0x0 },
            PDU_NOT_AVAILABLE,
            self.uptime.to_le_bytes()[0],
            self.uptime.to_le_bytes()[1],
            self.uptime.to_le_bytes()[2],
            self.uptime.to_le_bytes()[3],
        ])
        .build()
    }

    pub(crate) fn into_error(self) -> Result<(), J1939UnitError> {
        match self.state {
            State::Nominal => Ok(()),
            State::Ident => Ok(()),
            State::FaultyGenericError => Err(J1939UnitError::BusError),
            State::FaultyBusError => Err(J1939UnitError::BusError),
        }
    }
}

impl std::fmt::Display for VecraftStatusMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seconds = self.uptime % 60;
        let minutes = (self.uptime / 60) % 60;
        let hours = self.uptime / 3600;

        write!(
            f,
            "Status: {} Motion: {} Uptime: {:02}:{:02}:{:02}",
            self.state,
            if self.locked { "Locked" } else { "Unlocked" },
            hours,
            minutes,
            seconds
        )
    }
}
