use std::time::{Duration, Instant};

pub mod encoder;
pub mod engine;
pub mod fuzzer;
pub mod hydraulic;
pub mod inclino;
pub mod inspector;
pub mod probe;
pub mod vcu;
pub(super) mod vecraft;
pub mod volvo_ems;
mod volvo_vecu;

pub struct NetDriverContextDetail {
    /// Last message sent.
    tx_last_message: Option<crate::core::ObjectMessage>,
    /// Last time a message was received.
    rx_last: Instant,
    /// Last message received.
    rx_last_message: Option<crate::core::ObjectMessage>,
}

impl NetDriverContextDetail {
    /// Check if the last message was sent within a timeout.
    fn is_rx_timeout(&self, timeout: Duration) -> bool {
        self.rx_last.elapsed() > timeout
    }

    /// Mark the last time a message was received.
    fn rx_mark(&mut self) {
        self.rx_last = Instant::now();
    }
}

impl Default for NetDriverContextDetail {
    fn default() -> Self {
        Self {
            tx_last_message: None,
            rx_last: Instant::now(),
            rx_last_message: None,
        }
    }
}

#[derive(Default, Clone)]
pub struct NetDriverContext {
    detail: std::sync::Arc<std::sync::Mutex<NetDriverContextDetail>>,
}

impl NetDriverContext {
    pub fn inner(&self) -> std::sync::MutexGuard<NetDriverContextDetail> {
        self.detail.lock().unwrap()
    }

    /// Check if the last message was sent within a timeout.
    pub fn is_rx_timeout(&self, timeout: Duration) -> bool {
        self.detail.lock().unwrap().is_rx_timeout(timeout)
    }

    /// Mark the last time a message was received.
    pub fn rx_mark(&self) {
        self.detail.lock().unwrap().rx_mark();
    }

    pub fn set_tx_last_message(&self, message: crate::core::ObjectMessage) {
        self.detail.lock().unwrap().tx_last_message = Some(message);
    }

    pub fn set_rx_last_message(&self, message: crate::core::ObjectMessage) {
        self.detail.lock().unwrap().rx_last_message = Some(message);
    }

    // pub fn tx_last_message(&self) -> Option<crate::core::ObjectMessage> {
    //     self.detail.lock().unwrap().tx_last_message
    // }

    // pub fn rx_last_message(&self) -> Option<crate::core::ObjectMessage> {
    //     self.detail.lock().unwrap().rx_last_message
    // }
}

#[derive(Debug)]
pub enum J1939UnitError {
    /// Unit has not sent a message in a while.
    MessageTimeout,
    /// Unit has an invalid configuration.
    InvalidConfiguration,
    /// Version mismatch.
    VersionMismatch,
    /// Unit communication error.
    BusError,
    /// Unit has an i/o error.
    IOError(std::io::Error),
}

impl std::fmt::Display for J1939UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MessageTimeout => "communication timeout",
                Self::InvalidConfiguration => "invalid configuration",
                Self::VersionMismatch => "version mismatch",
                Self::BusError => "bus error",
                Self::IOError(error) => return write!(f, "i/o error: {}", error),
            }
        )
    }
}

impl From<std::io::Error> for J1939UnitError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}

impl std::error::Error for J1939UnitError {}

pub trait J1939Unit: Send + Sync {
    /// Get the vendor of the unit.
    fn vendor(&self) -> &'static str;

    /// Get the product of the unit.
    fn product(&self) -> &'static str;

    /// Get the name of the unit.
    fn name(&self) -> String {
        format!("{}:{}", self.vendor(), self.product())
    }

    /// Get the destination address of the unit.
    fn destination(&self) -> u8;

    /// Get the source address of the unit.
    fn source(&self) -> u8;

    /// Setup the unit.
    ///
    /// This method will be called to setup the unit. This method should be non-blocking and should
    /// only perform asynchronous I/O operations. This method is optional and may be a no-op.
    fn setup(
        &self,
        _ctx: &mut NetDriverContext,
        _tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        Ok(())
    }

    /// Teardown the unit.
    ///
    /// This method will be called to teardown the unit. This method should be non-blocking and should
    /// only perform asynchronous I/O operations. This method is optional and may be a no-op.
    fn teardown(
        &self,
        _ctx: &mut NetDriverContext,
        _tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        Ok(())
    }

    /// Try to accept a message from the router.
    ///
    /// This method will try to accept a message from the router. If the router has a message
    /// available, the message will be parsed and the unit will be updated accordingly. This
    /// method should be non-blocking and should only perform asynchronous I/O operations.
    ///
    /// It is advised to use the `try_accept` method, as opposed to the `tick` method, to handle
    /// unit setup and teardown. Do not perform any actual work in the `setup` and `teardown`
    /// methods, as they can cause network congestion and slow down the system.
    fn try_recv(
        &self,
        ctx: &mut NetDriverContext,
        frame: &j1939::Frame,
        signal_tx: crate::runtime::SignalSender,
    ) -> Result<(), J1939UnitError>;

    /// Trigger the unit manually.
    ///
    /// This method will be called to trigger the unit manually. This method should be non-blocking
    /// and should only perform asynchronous I/O operations.
    ///
    /// This method is optional and may be a no-op.
    fn trigger(
        &self,
        _ctx: &mut NetDriverContext,
        _tx_queue: &mut Vec<j1939::Frame>,
        _object: &crate::core::Object,
    ) -> Result<(), J1939UnitError> {
        Ok(())
    }

    fn tick(
        &self,
        _ctx: &mut NetDriverContext,
        _tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        Ok(())
    }
}
