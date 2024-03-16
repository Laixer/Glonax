use j1939::{Frame, FrameBuilder, IdBuilder, PDU_NOT_AVAILABLE, PGN};

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

        if frame.pdu()[2] != 0xff {
            if frame.pdu()[2] == 0x0 {
                ident_on = Some(false);
            } else if frame.pdu()[2] == 0x1 {
                ident_on = Some(true);
            }
        }

        if frame.pdu()[3] != 0xff && frame.pdu()[3] == 0x69 {
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
        .copy_from_slice(&[b'Z', b'C', 0xff, 0xff]);

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

// TODO: Remove the lock field.
pub struct VecraftStatusMessage {
    /// ECU status
    pub state: State,
    /// Motion lock
    pub locked: bool,
    /// Uptime
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
}

impl std::fmt::Display for VecraftStatusMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Status: {} Motion: {} Uptime: {}",
            self.state,
            if self.locked { "Locked" } else { "Unlocked" },
            self.uptime
        )
    }
}
