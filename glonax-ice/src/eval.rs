use tokio::io::{AsyncRead, AsyncWrite};

use crate::{DeviceInfo, PayloadType, Session, SessionError};

/// This is our local device address.
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
    pub async fn diagnose(&mut self) -> Result<(), SessionError> {
        Evaluation::new(&mut self.session).diagnose_test().await
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
    /// Quick probe test to determine if device is alive.
    pub async fn probe_test(&mut self) -> Result<(), SessionError> {
        self.session.add_payload_mask(PayloadType::DeviceInfo);

        debug!("Wait for a device announcement ...");

        let frame = self.session.accept().await?;
        let _: DeviceInfo = frame.get(6).unwrap();

        self.session.clear_payload_masks();

        debug!("Announce local device on network ...");

        self.session.announce_device().await?;

        Ok(())
    }

    /// Running session diagnostics.
    ///
    /// Try some basic tests to see whats going on with the device.
    pub async fn diagnose_test(&mut self) -> Result<(), SessionError> {
        info!("Running diagnostics on device");
        info!("Waiting to receive data...");

        self.probe_test().await?;

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
