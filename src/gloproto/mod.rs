// Copyright (C) 2021 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

// TODO:
// - Rename sugar to packet
// - Checksum
// - Sequence
// - Order evition

mod frame;
mod session;
mod sugar;

pub use session::Session;
pub use sugar::Sugar;

use std::convert::{TryFrom, TryInto};

use self::frame::Frame;

/// Protocol version.
pub const PROTOCOL_VERSION: u8 = 0x3;

#[derive(Debug, PartialEq)]
pub enum Command {
    CmdInfo = 0x80,   /* Command for information */
    CmdBoot = 0x81,   /* Command boot firmware */
    CmdIdle = 0x91,   /* Command ping */
    CmdCustom = 0xd0, /* Command custom */
    CmdError = 0xfa,  /* Command error report */
}

impl TryFrom<u8> for Command {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Command::CmdInfo as u8 => Ok(Command::CmdInfo),
            x if x == Command::CmdBoot as u8 => Ok(Command::CmdBoot),
            x if x == Command::CmdIdle as u8 => Ok(Command::CmdIdle),
            x if x == Command::CmdCustom as u8 => Ok(Command::CmdCustom),
            x if x == Command::CmdError as u8 => Ok(Command::CmdError),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TransmissionMode {
    Notify = 0x2,
    Request = 0x3,
    Response = 0x5,
}

impl TryFrom<u8> for TransmissionMode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == TransmissionMode::Notify as u8 => Ok(TransmissionMode::Notify),
            x if x == TransmissionMode::Request as u8 => Ok(TransmissionMode::Request),
            x if x == TransmissionMode::Response as u8 => Ok(TransmissionMode::Response),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum ProtocolError {
    TimeOut = 0,
    NoData = -1,
    InvalidProtocol = -2,
    VersionMismatch = -3,
    InvalidCommand = -4,
    InvalidSubCommand = -5,
    InvalidTransmissionMode = -6,
    InvalidParameter = -10,
    InvalidPayloadSize = -11,
}

impl TryFrom<i8> for ProtocolError {
    type Error = ();

    fn try_from(v: i8) -> Result<Self, Self::Error> {
        match v {
            x if x == ProtocolError::TimeOut as i8 => Ok(ProtocolError::TimeOut),
            x if x == ProtocolError::NoData as i8 => Ok(ProtocolError::NoData),
            x if x == ProtocolError::InvalidProtocol as i8 => Ok(ProtocolError::InvalidProtocol),
            x if x == ProtocolError::VersionMismatch as i8 => Ok(ProtocolError::VersionMismatch),
            x if x == ProtocolError::InvalidCommand as i8 => Ok(ProtocolError::InvalidCommand),
            x if x == ProtocolError::InvalidSubCommand as i8 => {
                Ok(ProtocolError::InvalidSubCommand)
            }
            x if x == ProtocolError::InvalidTransmissionMode as i8 => {
                Ok(ProtocolError::InvalidTransmissionMode)
            }
            x if x == ProtocolError::InvalidParameter as i8 => Ok(ProtocolError::InvalidParameter),
            x if x == ProtocolError::InvalidPayloadSize as i8 => {
                Ok(ProtocolError::InvalidPayloadSize)
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Header {
    /// Protocol version. Should match on both sides.
    version: u8,
    /// Optional flags.
    flags: u8,
    /// The command to carry out.
    pub ty: Command,
    /// Message transmission mode.
    pub tx_mod: TransmissionMode,
}

impl Header {
    fn from_tuple((version, flags, ty, tx_mod): (u8, u8, u8, u8)) -> Result<Self, ()> {
        Ok(Self {
            version,
            flags,
            ty: ty.try_into()?,
            tx_mod: tx_mod.try_into()?,
        })
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum Body {
    Error(i8),
    Info {
        unique_id: u16,
        firmware_major: u8,
        firmware_minor: u8,
    },
    Custom {
        subcommand: u16,
        flags: u16,
        payload: Vec<u8>,
    },
}

/// `FrameBuilder` can construct a new frame.
///
/// This is the recommended way to construct new frames.
/// All constructed frames are spec compliant.
pub struct FrameBuilder(Frame);

impl FrameBuilder {
    /// The default frame will be an idle word.
    ///
    /// It is recommended to use either `from_command` or `from_body`.
    pub fn new() -> Self {
        Self::from_command(Command::CmdIdle, TransmissionMode::Notify)
    }

    /// Create frame from command.
    pub fn from_command(command: Command, mode: TransmissionMode) -> Self {
        Self(Frame {
            header: Header {
                version: PROTOCOL_VERSION,
                flags: 0,
                ty: command,
                tx_mod: mode,
            },
            body: None,
        })
    }

    /// Create frame from body.
    ///
    /// This is the recommended way to construct a frame. Frames
    /// which originate from a body are most likely valid.
    pub fn from_body(body: Body) -> Self {
        Self(Frame {
            header: Header {
                version: PROTOCOL_VERSION,
                flags: 0,
                ty: match &body {
                    Body::Error(_) => Command::CmdError,
                    Body::Info { .. } => Command::CmdInfo,
                    Body::Custom { .. } => Command::CmdCustom,
                },
                tx_mod: TransmissionMode::Request,
            },
            body: Some(body),
        })
    }

    /// Set transmission mode to `Request`.
    pub fn set_request(mut self) -> Self {
        self.0.header.tx_mod = TransmissionMode::Request;
        self
    }

    /// Set transmission mode to `Notify`.
    pub fn set_notify(mut self) -> Self {
        self.0.header.tx_mod = TransmissionMode::Notify;
        self
    }

    /// Validate internal frame.
    ///
    /// Catch any invalid configurations before they are
    /// constructed.
    fn validate(self) -> std::result::Result<Self, ()> {
        let mode = &self.0.header.tx_mod;
        match self.0.header.ty {
            Command::CmdInfo => Ok(self),
            Command::CmdBoot => match mode {
                TransmissionMode::Notify => Err(()),
                _ => Ok(self),
            },
            Command::CmdIdle => match mode {
                TransmissionMode::Notify => Ok(self),
                _ => Err(()),
            },
            Command::CmdCustom => Ok(self),
            Command::CmdError => match mode {
                TransmissionMode::Request => Err(()),
                _ => Ok(self),
            },
        }
    }

    /// Construct frame.
    pub fn build(self) -> std::result::Result<Frame, ()> {
        Ok(self.validate()?.0)
    }
}

impl Default for Frame {
    /// The default frame will be an idle word.
    fn default() -> Self {
        FrameBuilder::new().build().unwrap()
    }
}
