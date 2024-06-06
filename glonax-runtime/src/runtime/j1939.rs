use std::{
    future::Future,
    time::{Duration, Instant},
};

use crate::core::{Object, ObjectMessage};

use super::SignalSender;

pub struct NetDriverContextDetail {
    /// Last message sent.
    tx_last_message: Option<ObjectMessage>,
    /// Last time a message was received.
    rx_last: Instant,
    /// Last message received.
    rx_last_message: Option<ObjectMessage>,
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

    pub fn set_tx_last_message(&self, message: ObjectMessage) {
        self.detail.lock().unwrap().tx_last_message = Some(message);
    }

    pub fn set_rx_last_message(&self, message: ObjectMessage) {
        self.detail.lock().unwrap().rx_last_message = Some(message);
    }

    pub fn tx_last_message(&self) -> Option<ObjectMessage> {
        self.detail.lock().unwrap().tx_last_message.clone()
    }

    pub fn rx_last_message(&self) -> Option<ObjectMessage> {
        self.detail.lock().unwrap().rx_last_message.clone()
    }
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

pub enum J1939UnitOk {
    /// Unit has queued a signal.
    SignalQueued,
    /// Unit has parsed a frame.
    FrameParsed,
    /// Unit has ignored a frame.
    FrameIgnored,
}

/// Represents a J1939 unit.
pub trait J1939Unit: Send + Sync {
    /// Get the vendor of the unit.
    fn vendor(&self) -> &'static str;

    /// Get the product of the unit.
    fn product(&self) -> &'static str;

    /// Get the name of the unit.
    ///
    /// The name is formatted as "{vendor}:{product}:0x{source}:0x{destination}".
    fn name(&self) -> String {
        format!(
            "{}:{}:0x{:X}:0x{:X}",
            self.vendor(),
            self.product(),
            self.source(),
            self.destination()
        )
    }

    /// Get the destination address of the unit.
    fn destination(&self) -> u8;

    /// Get the source address of the unit.
    fn source(&self) -> u8;

    /// Setup the unit.
    ///
    /// This method will be called to setup the unit. This method should be non-blocking and should
    /// only perform asynchronous I/O operations. This method is optional and may be a no-op.
    #[allow(unused_variables)]
    fn setup(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        Ok(())
    }

    /// Teardown the unit.
    ///
    /// This method will be called to teardown the unit. This method should be non-blocking and should
    /// only perform asynchronous I/O operations. This method is optional and may be a no-op.
    #[allow(unused_variables)]
    fn teardown(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
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
    ) -> Result<J1939UnitOk, J1939UnitError>;

    /// Trigger the unit manually.
    ///
    /// This method will be called to trigger the unit manually. This method should be non-blocking
    /// and should only perform asynchronous I/O operations.
    ///
    /// This method is optional and may be a no-op.
    #[allow(unused_variables)]
    fn trigger(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
        object: &Object,
    ) -> Result<(), J1939UnitError> {
        Ok(())
    }

    /// Perform a tick operation on the unit.
    ///
    /// This method will be called periodically to perform any necessary operations on the unit.
    /// This method should be non-blocking and should only perform asynchronous I/O operations.
    ///
    /// This method is optional and may be a no-op.
    #[allow(unused_variables)]
    fn tick(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        Ok(())
    }
}

pub trait NetworkService<Cnf> {
    /// Creates a new instance of the network service with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the network service.
    ///
    /// # Returns
    ///
    /// The newly created network service instance.
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    /// Sets up the network service.
    ///
    /// This method is called during the initialization of the network service.
    /// Implementations should perform any necessary setup operations here.
    ///
    /// # Returns
    ///
    /// A future that resolves when the setup is complete.
    fn setup(&mut self) -> impl Future<Output = ()> + Send {
        async {}
    }

    /// Tears down the network service.
    ///
    /// This method is called during the shutdown of the network service.
    /// Implementations should perform any necessary cleanup operations here.
    ///
    /// # Returns
    ///
    /// A future that resolves when the teardown is complete.
    fn teardown(&mut self) -> impl Future<Output = ()> + Send {
        async {}
    }

    /// Receives a signal from the network.
    ///
    /// This method is called when a signal is received from the network.
    /// Implementations should handle the received signal here.
    ///
    /// # Arguments
    ///
    /// * `signal_tx` - The sender for signals.
    ///
    /// # Returns
    ///
    /// A future that resolves when the signal has been processed.
    fn recv(&mut self, signal_tx: SignalSender) -> impl Future<Output = ()> + Send;

    /// Performs an action on each tick of the network service.
    ///
    /// This method is called on each tick of the network service.
    /// Implementations should perform any necessary actions here.
    ///
    /// # Returns
    ///
    /// A future that resolves when the action has been performed.
    fn on_tick(&mut self) -> impl Future<Output = ()> + Send;

    /// Performs an action in response to a command.
    ///
    /// This method is called when a command is received by the network service.
    /// Implementations should perform the necessary action based on the received command.
    ///
    /// # Arguments
    ///
    /// * `object` - The object representing the received command.
    ///
    /// # Returns
    ///
    /// A future that resolves when the action has been performed.
    fn on_command(&mut self, object: &Object) -> impl Future<Output = ()> + Send;
}
