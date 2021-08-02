use std::{
    convert::TryFrom,
    io::{Read, Write},
    usize,
};

use crate::{
    common::Ring,
    gloproto::{sugar::PacketError, Body, ProtocolError},
};

use super::{Command, Frame, FrameBuilder, Sugar, TransmissionMode};

pub struct PeerInfo {
    /// Peer ID.
    id: u16,
    /// Firmware version.
    firmware_version: (u8, u8),
}

impl std::fmt::Debug for PeerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Remote peer ID: {:x}; Firmware version: {}.{}",
            self.id, self.firmware_version.0, self.firmware_version.1
        )
    }
}

#[derive(Debug)]
pub struct Stats {
    /// Total number of ingress frames including failures.
    pub rx_count: usize,
    /// Number of malformed ingress frames.
    pub rx_failure: usize,
    /// Total number egress frames.
    pub tx_count: usize,
    /// Number of malformed egress frames.
    pub tx_failure: usize,
}

impl Stats {
    /// Create new empty statistics.
    pub fn new() -> Self {
        Self {
            rx_count: 0,
            rx_failure: 0,
            tx_count: 0,
            tx_failure: 0,
        }
    }

    /// Calculate ingress faillure rate in percentage.
    pub fn rx_faillure_rate(&self) -> usize {
        if self.rx_count > 0 {
            (self.rx_failure / self.rx_count) * 100
        } else {
            0
        }
    }

    /// Calculate egress faillure rate in percentage.
    pub fn tx_faillure_rate(&self) -> usize {
        if self.tx_count > 0 {
            (self.tx_failure / self.tx_count) * 100
        } else {
            0
        }
    }

    /// Reset statistics.
    pub fn reset(&mut self) {
        *self = Self::new()
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SessionOptions {
    /// Retry on I/O timeout.
    ///
    /// If this option is enabled the session manager
    /// will try to read or write again if the underlaying
    /// I/O device reports a timeout. Defaults to `false`.
    pub retry_timeout: bool,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            retry_timeout: false,
        }
    }
}

pub struct Session<T> {
    /// Read ring buffer.
    buf: Ring<T>,
    /// Peer (device) information.
    pub peer_info: Option<PeerInfo>,
    /// Session statistics.
    pub stats: Stats,
    /// Session options.
    options: SessionOptions,
}

enum FetchError {
    /// Found end of file/stream.
    EndOfFile,
    /// Frame was not complete.
    Incomplete,
    /// Frame failed to parse.
    ParseError,
    /// I/O timeout.
    IoTimedOut,
    /// Generic I/O error.
    IoError(std::io::Error),
}

impl<T> Session<T> {
    /// Construct new session.
    ///
    /// The returned session may not be initialized yet.
    pub fn new(inner: T) -> Session<T> {
        Self {
            buf: Ring::new(inner),
            peer_info: None,
            stats: Stats::new(),
            options: SessionOptions::default(),
        }
    }

    /// Gets a reference to the inner device.
    #[inline]
    pub fn get_ref(&self) -> &T {
        self.buf.get_ref()
    }

    /// Gets a mutable reference to the inner device.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.buf.get_mut()
    }

    // TODO: Maybe replace by more generic `is_alive`.
    /// Check if the device is initialized.
    pub fn is_initialized(&self) -> bool {
        if let Some(peer) = &self.peer_info {
            peer.id > 0
        } else {
            false
        }
    }
}

impl<T: Write> Session<T> {
    #[inline]
    fn write_buf(&mut self, buf: &[u8]) {
        if let Err(_) = self.get_mut().write(buf) {
            self.stats.tx_failure += 1;
        }
        self.stats.tx_count += 1;
    }

    /// Send idle word.
    ///
    /// This packet does not contain any actual data. Its purpose is to
    /// keep the channel active. Idle words should be send in idle time.
    #[allow(dead_code)]
    #[inline]
    fn notify_idle_word(&mut self) {
        self.write_buf(&Frame::default().to_bytes());
    }

    /// Request device information.
    fn request_info(&mut self) {
        let frame = FrameBuilder::from_command(Command::CmdInfo, TransmissionMode::Request)
            .build()
            .unwrap();
        self.write_buf(&frame.to_bytes());
    }

    /// Request pulse pin.
    ///
    /// This will instruct a PWM port to set the voltage to
    /// the provided value.
    pub fn request_pulse_pin(&mut self, pin: u8, value: i16) {
        let frame = FrameBuilder::from_body(Sugar::PulsePin(pin, value).to_body())
            .build()
            .unwrap();
        self.write_buf(&frame.to_bytes());
    }

    /// Send a request packet.
    pub fn request(&mut self, packet: Sugar) {
        let frame = FrameBuilder::from_body(packet.to_body()).build().unwrap();
        self.write_buf(&frame.to_bytes());
    }
}

impl<T: Read> Session<T> {
    fn fetch_next(&mut self) -> Result<Frame, FetchError> {
        let mut local_buf = [0; 512];

        let sread = self
            .buf
            .read(&mut local_buf)
            .map_err(|e: std::io::Error| match e.kind() {
                std::io::ErrorKind::TimedOut => FetchError::IoTimedOut,
                _ => FetchError::IoError(e),
            })?;
        if sread == 0 {
            return Err(FetchError::EndOfFile);
        }

        // TODO: Validate frame.
        // TODO: Fix all unwraps.
        // TODO: Check version
        match Frame::parse(&local_buf[..sread]) {
            Ok(frame) => {
                let size_left = frame.0.len();
                let frame = frame.1;
                self.buf.advance_return(size_left).unwrap();
                self.stats.rx_count += 1;
                Ok(frame)
            }
            Err(nom::Err::Incomplete(_)) => {
                if self
                    .buf
                    .fill_buf()
                    .map_err(|e: std::io::Error| match e.kind() {
                        std::io::ErrorKind::TimedOut => FetchError::IoTimedOut,
                        _ => FetchError::IoError(e),
                    })?
                    == 0
                {
                    Err(FetchError::EndOfFile)
                } else {
                    Err(FetchError::Incomplete)
                }
            }
            Err(_) => {
                self.stats.rx_failure += 1;
                Err(FetchError::ParseError)
            }
        }
    }

    /// Return next `Sugar` message.
    ///
    /// If no `Sugar` message was found `None` is returned.
    /// This method can block if the underlaying reader device
    /// blocks on read calls.
    pub fn next(&mut self) -> Option<Sugar> {
        loop {
            match self.fetch_next() {
                Ok(frame) => {
                    if frame.header.ty == Command::CmdIdle {
                        return None;
                    }

                    match frame.body {
                        Some(Body::Error(e)) => {
                            match e {
                                e if e > -19 => {
                                    log::error!("{:?}", ProtocolError::try_from(e).unwrap())
                                }
                                e if e > -128 => {
                                    log::error!("{:?}", PacketError::try_from(e).unwrap())
                                }
                                _ => log::error!("Unkown error code: {}", e),
                            }
                            self.stats.rx_failure += 1
                        }
                        Some(Body::Info {
                            unique_id,
                            firmware_major,
                            firmware_minor,
                        }) => {
                            self.peer_info = Some(PeerInfo {
                                id: unique_id,
                                firmware_version: (firmware_major, firmware_minor),
                            });
                            return None;
                        }
                        Some(Body::Custom {
                            subcommand,
                            flags,
                            payload,
                        }) => {
                            if !payload.is_empty() {
                                match Sugar::parse(subcommand, flags, &payload) {
                                    Ok(s) => return Some(s),
                                    Err(_) => {
                                        self.stats.rx_failure += 1;
                                        return None;
                                    }
                                }
                            } else {
                                return None;
                            }
                        }
                        None => self.stats.rx_failure += 1, // TODO: Should this not be a failure?
                    }
                }
                Err(FetchError::Incomplete) => continue,
                Err(FetchError::EndOfFile) => return None,
                Err(FetchError::IoError(e)) => {
                    log::error!("I/O error: {}", e);
                    return None;
                }
                Err(FetchError::IoTimedOut) => {
                    if self.options.retry_timeout {
                        continue;
                    }
                    return None;
                }
                Err(FetchError::ParseError) => {
                    log::error!("Parser error");
                    return None;
                }
            }
        }
    }
}

impl<T: Read + Write> Session<T> {
    /// Wait until session is initialized.
    ///
    /// This method will block until the session is initialized.
    pub fn wait_for_init(&mut self) {
        loop {
            self.request_info();
            self.next();
            if self.is_initialized() {
                break;
            }
        }
    }

    /// Construct and initialize new session.
    ///
    /// This method will block until the session is initialized.
    pub fn open(inner: T) -> Session<T> {
        let mut session = Self::new(inner);
        session.wait_for_init();
        session
    }
}
