use std::convert::TryFrom;

use self::{double_cursor::DoubleCursor, stats::Stats};

mod double_cursor;
pub mod stats;

#[macro_use]
extern crate log;

const MAGIC: [u8; 2] = [0xc5, 0x34];
const ICE_PROTOCOL_VERSION: u8 = 5;

enum AddressFamily {
    Broadcast,
    Unicast(u16),
}

pub enum PayloadType {
    /// Device information.
    DeviceInfo = 0x10,
    /// Solenoid control.
    SolenoidControl = 0x11,
    /// Temperature type.
    MeasurementTemperature = 0x12,
    /// Acceleration type.
    MeasurementAcceleration = 0x13,
    /// Angular velocity type.
    MeasurementAngularVelocity = 0x14,
    /// Direction type.
    MeasurementDirection = 0x15,
}

impl TryFrom<u8> for PayloadType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            v if v == Self::DeviceInfo as u8 => Ok(Self::DeviceInfo),
            v if v == Self::SolenoidControl as u8 => Ok(Self::SolenoidControl),
            v if v == Self::MeasurementTemperature as u8 => Ok(Self::MeasurementTemperature),
            v if v == Self::MeasurementAcceleration as u8 => Ok(Self::MeasurementAcceleration),
            v if v == Self::MeasurementAngularVelocity as u8 => {
                Ok(Self::MeasurementAngularVelocity)
            }
            v if v == Self::MeasurementDirection as u8 => Ok(Self::MeasurementDirection),
            _ => Err(()),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Packet {
    version: u8,
    pub payload_type: u8,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Vector3x16 {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct DeviceInfo {
    address: u16,
    version: u8,
    status: u8,
}

impl std::fmt::Debug for DeviceInfo {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = self.address;
        write!(
            fmt,
            "Device announcement: Address: {} Version: {}.{}",
            address,
            (self.version >> 4),
            (self.version & !0xf0),
        )
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct SolenoidControl {
    id: u8,
    value: i16,
}

#[derive(Debug, Clone, Copy)]
pub enum FrameError {
    InvalidMagic(usize),
    InvalidChecksum,
    IncompatibleVersion,
}

pub struct Frame {
    buffer: [u8; 14],
}

impl Frame {
    const SIZE: usize = 14; // TODO: Calculate this.

    fn new() -> Self {
        Self {
            buffer: [0; Frame::SIZE],
        }
    }

    // TODO: AsRef<[u8]>
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn from_slice(slice: &[u8]) -> Self {
        let mut frame = Self::new();
        frame.buffer.copy_from_slice(slice);
        frame
    }

    fn set<T>(&self, offset: usize, value: T) -> std::result::Result<(), ()> {
        if offset >= self.buffer.len() {
            Err(())
        } else {
            unsafe { std::ptr::write(self.buffer[offset..].as_ptr() as *mut _, value) };
            Ok(())
        }
    }

    pub fn get<T>(&self, offset: usize) -> std::result::Result<T, ()> {
        if offset >= self.buffer.len() {
            Err(())
        } else {
            Ok(unsafe { std::ptr::read(self.buffer[offset..].as_ptr() as *const _) })
        }
    }

    pub fn is_broadcast(&self) -> bool {
        self.address() == u16::MAX
    }

    pub fn address(&self) -> u16 {
        self.get(2).unwrap()
    }

    /// Calculate checksum over frame body.
    fn calc_checksum(&self) -> u16 {
        let crc = crc::Crc::<u16>::new(&crc::CRC_16_IBM_3740);
        crc.checksum(&self.buffer[4..12])
    }

    #[inline]
    pub fn packet(&self) -> Packet {
        self.get(4).unwrap()
    }

    fn is_valid(&self) -> std::result::Result<(), FrameError> {
        if self.buffer[0] != MAGIC[0] {
            return Err(FrameError::InvalidMagic(0));
        } else if self.buffer[1] != MAGIC[1] {
            return Err(FrameError::InvalidMagic(1));
        }

        let packet_sum: u16 = self.get(12).unwrap();
        if self.calc_checksum() != packet_sum {
            return Err(FrameError::InvalidChecksum);
        }

        if self.packet().version != ICE_PROTOCOL_VERSION {
            return Err(FrameError::IncompatibleVersion);
        }

        Ok(())
    }
}

struct FrameBuilder {
    frame: Frame,
}

impl FrameBuilder {
    fn new() -> Self {
        Self {
            frame: Frame::new(),
        }
    }

    fn set_address(&mut self, address: AddressFamily) {
        let address = match address {
            AddressFamily::Broadcast => u16::MAX,
            AddressFamily::Unicast(address) => address,
        };
        self.frame.set(2, address).unwrap();
    }

    fn set_payload<T>(&mut self, payload: T, payload_type: PayloadType) {
        self.frame.set(6, payload).unwrap();

        self.frame
            .set(
                4,
                Packet {
                    version: ICE_PROTOCOL_VERSION,
                    payload_type: payload_type as u8,
                },
            )
            .unwrap();
    }

    fn build(mut self) -> Frame {
        self.frame.buffer[0] = MAGIC[0];
        self.frame.buffer[1] = MAGIC[1];

        self.frame.set(12, self.frame.calc_checksum()).unwrap();
        self.frame
    }
}

pub enum SessionError {
    /// Packet was not send to this address.
    SpuriousAddress,
    /// Frame was not complete.
    Incomplete,
    /// Frame was not found in buffer.
    InvalidData,
    /// Frame parse errror.
    FrameParseError(FrameError),
    /// I/O error in the underlaying device.
    IoError(std::io::Error),
}

impl std::fmt::Debug for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SpuriousAddress => write!(f, "packet was not send to this address"),
            Self::Incomplete => write!(f, "frame was not complete"),
            Self::InvalidData => write!(f, "frame was not found in buffer"),
            Self::FrameParseError(arg0) => f.debug_tuple("FrameParseError").field(arg0).finish(),
            Self::IoError(arg0) => f.debug_tuple("IoError").field(arg0).finish(),
        }
    }
}

pub struct Session<T> {
    /// Inner device.
    inner: T,
    /// Session statistics.
    pub stats: Stats,
    /// Local address.
    pub address: u16,
    /// Reading buffer.
    buffer: DoubleCursor<[u8; 4096]>,
}

impl<T> Session<T> {
    /// Gets a reference to the inner device.
    #[inline]
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the inner device.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: std::io::Read> Session<T> {
    /// Construct new session.
    pub fn new(inner: T, address: u16) -> Self {
        Self {
            inner,
            stats: Stats::new(),
            address,
            buffer: DoubleCursor::new([0u8; 4096]),
        }
    }

    /// Return next `Frame`.
    ///
    /// This method can block if the underlaying reader device
    /// blocks on read calls.
    pub fn next(&mut self) -> std::result::Result<Frame, SessionError> {
        let taken = self
            .inner
            .read(self.buffer.allocate())
            .map_err(|err| SessionError::IoError(err))?;
        self.buffer.fill(taken);

        if let Some(offset) = self.buffer.buffer().iter().position(|&b| b == MAGIC[0]) {
            self.buffer.consume(offset);

            if self.buffer.len() >= Frame::SIZE {
                let frame = Frame::from_slice(&self.buffer.buffer()[..Frame::SIZE]);
                self.buffer.consume(Frame::SIZE);

                self.stats.rx_count += 1;

                match frame.is_valid() {
                    Ok(_) => {
                        if frame.is_broadcast() || frame.address() == self.address {
                            Ok(frame)
                        } else {
                            Err(SessionError::SpuriousAddress)
                        }
                    }
                    Err(e) => {
                        self.stats.rx_failure += 1;
                        Err(SessionError::FrameParseError(e))
                    }
                }
            } else {
                Err(SessionError::Incomplete)
            }
        } else {
            Err(SessionError::Invalid)
        }
    }

    /// Return next `Frame`.
    ///
    /// If no `Frame` message was found or the frame was invalid.
    /// then this method will wait for the next frame. Therefore
    /// this method is guaranteed to return only valid frames.
    ///
    /// This method can block if the underlaying reader device
    /// blocks on read calls.
    pub fn accept(&mut self) -> Frame {
        loop {
            match self.next() {
                Ok(frame) => {
                    if frame.is_broadcast() || frame.address() == self.address {
                        break frame;
                    }
                }
                Err(SessionError::Incomplete) => continue,
                Err(e) => warn!("{:?}", e),
            }
        }
    }
}

impl<T: std::io::Write> Session<T> {
    /// Write raw frame to the inner device.
    ///
    /// Any IO errors will propagate upwards.
    fn write_frame(&mut self, frame: Frame) -> std::result::Result<(), SessionError> {
        self.inner.write(frame.buffer()).map_err(|err| {
            self.stats.tx_failure += 1;
            SessionError::IoError(err)
        })?;
        self.stats.tx_count += 1;

        // TODO: HACK: README: XXX: We deliberately delay the write
        // operation so that the MCU gets enough time to process the
        // imcoming packet.
        std::thread::sleep(std::time::Duration::from_millis(5));
        Ok(())
    }

    /// Announce this device on the network.
    pub fn announce_device(&mut self) -> std::result::Result<(), SessionError> {
        let mut builder = FrameBuilder::new();

        builder.set_address(AddressFamily::Broadcast);
        builder.set_payload(
            DeviceInfo {
                address: self.address,
                version: 33, // TODO
                status: 0,
            },
            PayloadType::DeviceInfo,
        );

        self.write_frame(builder.build())
    }

    /// Dispatch valve control message.
    pub fn dispatch_valve_control(
        &mut self,
        id: u8,
        value: i16,
    ) -> std::result::Result<(), SessionError> {
        let mut builder = FrameBuilder::new();

        builder.set_address(AddressFamily::Unicast(0x7));
        builder.set_payload(SolenoidControl { id, value }, PayloadType::SolenoidControl);

        self.write_frame(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MemoryDevice {
        buf: [u8; Frame::SIZE],
    }

    impl MemoryDevice {
        fn new() -> Self {
            Self {
                buf: [0; Frame::SIZE],
            }
        }
    }

    impl std::io::Read for MemoryDevice {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            buf[..self.buf.len()].copy_from_slice(&self.buf[..]);
            Ok(self.buf.len())
        }
    }

    impl std::io::Write for MemoryDevice {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buf.copy_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn session_write_read() {
        const ADDR: u16 = 0x15;

        let device = MemoryDevice::new();

        let mut session = Session::new(device, ADDR);
        session.announce_device().unwrap();
        let frame = session.accept();

        assert!(frame.is_broadcast());
        assert_eq!(frame.address(), u16::MAX);

        let dev_info: DeviceInfo = frame.get(6).unwrap();
        let address = dev_info.address;

        assert_eq!(address, ADDR);
    }
}
