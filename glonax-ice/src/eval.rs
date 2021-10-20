use tokio::io::{AsyncRead, AsyncWrite};

use crate::{DeviceInfo, PayloadType, Session, SessionError};

/// This is the local device address for the container session.
const DEVICE_ADDR: u16 = 0x60;

pub struct ContainerSession<T> {
    session: Session<T>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> ContainerSession<T> {
    pub fn new(device: T) -> Self {
        Self {
            session: Session::new(device, DEVICE_ADDR),
        }
    }

    /// Running diagnostics.
    #[inline]
    pub async fn diagnose(&mut self) -> Result<(), SessionError> {
        Evaluation::new(&mut self.session).diagnose_test().await
    }
}

pub struct ScanResult {
    pub address: u16,
    pub version: u8,
    pub status: u8,
}

impl std::fmt::Debug for ScanResult {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let address = self.address;
        write!(
            fmt,
            "Address: {}; Version: {}.{}; Status: {}",
            address,
            (self.version >> 4),
            (self.version & !0xf0),
            self.status
        )
    }
}

pub struct Evaluation<'a, T> {
    session: &'a mut Session<T>,
}

impl<'a, T> Evaluation<'a, T> {
    pub fn new(session: &'a mut Session<T>) -> Self {
        Self { session }
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> Evaluation<'_, T> {
    /// Scan the network for devices.
    ///
    /// Return all devices found on the network.
    pub async fn network_scan(&mut self) -> Result<ScanResult, SessionError> {
        self.session.add_payload_mask(PayloadType::DeviceInfo);

        debug!("Wait for a device announcement ...");

        let frame = self.session.accept().await?;
        let device_info: DeviceInfo = frame.get(6).unwrap();

        self.session.clear_payload_masks();

        debug!("Announce local device on network ...");

        self.session.announce_device().await?;

        Ok(ScanResult {
            address: device_info.address,
            version: device_info.version,
            status: device_info.status,
        })
    }

    /// Running session diagnostics.
    ///
    /// Try some basic tests to see whats going on with the device.
    pub async fn diagnose_test(&mut self) -> Result<(), SessionError> {
        info!("Running diagnostics on device");
        info!("Waiting to receive data...");

        self.network_scan().await?;

        debug!("Testing 5 incoming packets ...");

        for i in 0..5 {
            match self.session.next().await {
                Ok(_) => info!("Found valid packet {}", i + 1),
                Err(e) => error!("Session fault: {:?}", e),
            }
        }

        debug!("Testing 5 outgoing packets ...");

        for i in 0..5 {
            match self.session.announce_device().await {
                Ok(_) => info!("Wrote packet {} to device", i + 1),
                Err(e) => error!("Session fault: {:?}", e),
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        Ok(())
    }
}
