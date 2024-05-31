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
pub mod vms;
pub mod volvo_ems;
mod volvo_vecu;

#[derive(Clone)]
pub enum NetDriver {
    KueblerEncoder(super::KueblerEncoder),
    KueblerInclinometer(super::KueblerInclinometer),
    VolvoD7E(super::VolvoD7E),
    HydraulicControlUnit(super::HydraulicControlUnit),
    VehicleManagementSystem(super::VehicleManagementSystem),
    VehicleControlUnit(super::VehicleControlUnit),
}

impl NetDriver {
    pub(crate) fn factory(
        vendor: &str,
        product: &str,
        destination: u8,
        source: u8,
    ) -> Result<Self, ()> {
        match (vendor, product) {
            (v, p)
                if v == crate::driver::KueblerEncoder::VENDOR
                    && p == crate::driver::KueblerEncoder::PRODUCT =>
            {
                Ok(NetDriver::KueblerEncoder(
                    crate::driver::KueblerEncoder::new(destination, source),
                ))
            }
            (v, p)
                if v == crate::driver::KueblerInclinometer::VENDOR
                    && p == crate::driver::KueblerInclinometer::PRODUCT =>
            {
                Ok(NetDriver::KueblerInclinometer(
                    crate::driver::KueblerInclinometer::new(destination, source),
                ))
            }
            (v, p)
                if v == crate::driver::VolvoD7E::VENDOR
                    && p == crate::driver::VolvoD7E::PRODUCT =>
            {
                Ok(NetDriver::VolvoD7E(crate::driver::VolvoD7E::new(
                    destination,
                    source,
                )))
            }
            (v, p)
                if v == crate::driver::HydraulicControlUnit::VENDOR
                    && p == crate::driver::HydraulicControlUnit::PRODUCT =>
            {
                Ok(NetDriver::HydraulicControlUnit(
                    crate::driver::HydraulicControlUnit::new(destination, source),
                ))
            }
            (v, p)
                if v == crate::driver::VehicleControlUnit::VENDOR
                    && p == crate::driver::VehicleControlUnit::PRODUCT =>
            {
                Ok(NetDriver::VehicleControlUnit(
                    crate::driver::VehicleControlUnit::new(destination, source),
                ))
            }
            _ => Err(()),
        }
    }
}

impl J1939Unit for NetDriver {
    const VENDOR: &'static str = "";
    const PRODUCT: &'static str = "";

    fn name(&self) -> String {
        match self {
            Self::KueblerEncoder(encoder) => encoder.name(),
            Self::KueblerInclinometer(inclinometer) => inclinometer.name(),
            Self::VolvoD7E(volvo) => volvo.name(),
            Self::HydraulicControlUnit(hydraulic) => hydraulic.name(),
            Self::VehicleManagementSystem(responder) => responder.name(),
            Self::VehicleControlUnit(vcu) => vcu.name(),
        }
    }

    fn destination(&self) -> u8 {
        match self {
            Self::KueblerEncoder(encoder) => encoder.destination(),
            Self::KueblerInclinometer(inclinometer) => inclinometer.destination(),
            Self::VolvoD7E(volvo) => volvo.destination(),
            Self::HydraulicControlUnit(hydraulic) => hydraulic.destination(),
            Self::VehicleManagementSystem(responder) => responder.destination(),
            Self::VehicleControlUnit(vcu) => vcu.destination(),
        }
    }

    fn source(&self) -> u8 {
        match self {
            Self::KueblerEncoder(encoder) => encoder.source(),
            Self::KueblerInclinometer(inclinometer) => inclinometer.source(),
            Self::VolvoD7E(volvo) => volvo.source(),
            Self::HydraulicControlUnit(hydraulic) => hydraulic.source(),
            Self::VehicleManagementSystem(responder) => responder.source(),
            Self::VehicleControlUnit(vcu) => vcu.source(),
        }
    }

    async fn setup(
        &self,
        ctx: &mut NetDriverContext,
        router: &crate::net::ControlNetwork,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => encoder.setup(ctx, router).await,
            Self::KueblerInclinometer(inclinometer) => inclinometer.setup(ctx, router).await,
            Self::VolvoD7E(volvo) => volvo.setup(ctx, router).await,
            Self::HydraulicControlUnit(hydraulic) => hydraulic.setup(ctx, router).await,
            Self::VehicleManagementSystem(responder) => responder.setup(ctx, router).await,
            Self::VehicleControlUnit(vcu) => vcu.setup(ctx, router).await,
        }
    }

    async fn teardown(
        &self,
        ctx: &mut NetDriverContext,
        network: &crate::net::ControlNetwork,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => encoder.teardown(ctx, network).await,
            Self::KueblerInclinometer(inclinometer) => inclinometer.teardown(ctx, network).await,
            Self::VolvoD7E(volvo) => volvo.teardown(ctx, network).await,
            Self::HydraulicControlUnit(hydraulic) => hydraulic.teardown(ctx, network).await,
            Self::VehicleManagementSystem(responder) => responder.teardown(ctx, network).await,
            Self::VehicleControlUnit(vcu) => vcu.teardown(ctx, network).await,
        }
    }

    async fn try_accept(
        &mut self,
        ctx: &mut NetDriverContext,
        network: &crate::net::ControlNetwork,
        signal_tx: crate::runtime::SignalSender,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => encoder.try_accept(ctx, network, signal_tx).await,
            Self::KueblerInclinometer(inclinometer) => {
                inclinometer.try_accept(ctx, network, signal_tx).await
            }
            Self::VolvoD7E(volvo) => volvo.try_accept(ctx, network, signal_tx).await,
            Self::HydraulicControlUnit(hydraulic) => {
                hydraulic.try_accept(ctx, network, signal_tx).await
            }
            Self::VehicleManagementSystem(responder) => {
                responder.try_accept(ctx, network, signal_tx).await
            }
            Self::VehicleControlUnit(vcu) => vcu.try_accept(ctx, network, signal_tx).await,
        }
    }

    fn trigger(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
        object: &crate::core::Object,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => encoder.trigger(ctx, tx_queue, object),
            Self::KueblerInclinometer(inclinometer) => inclinometer.trigger(ctx, tx_queue, object),
            Self::VolvoD7E(volvo) => volvo.trigger(ctx, tx_queue, object),
            Self::HydraulicControlUnit(hydraulic) => hydraulic.trigger(ctx, tx_queue, object),
            Self::VehicleManagementSystem(responder) => responder.trigger(ctx, tx_queue, object),
            Self::VehicleControlUnit(vcu) => vcu.trigger(ctx, tx_queue, object),
        }
    }

    fn tick(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => encoder.tick(ctx, tx_queue),
            Self::KueblerInclinometer(inclinometer) => inclinometer.tick(ctx, tx_queue),
            Self::VolvoD7E(volvo) => volvo.tick(ctx, tx_queue),
            Self::HydraulicControlUnit(hydraulic) => hydraulic.tick(ctx, tx_queue),
            Self::VehicleManagementSystem(responder) => responder.tick(ctx, tx_queue),
            Self::VehicleControlUnit(vcu) => vcu.tick(ctx, tx_queue),
        }
    }
}

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

pub trait J1939Unit {
    const VENDOR: &'static str;
    const PRODUCT: &'static str;

    /// Get the name of the unit.
    fn name(&self) -> String {
        format!("{}:{}", Self::VENDOR, Self::PRODUCT)
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
        _network: &crate::net::ControlNetwork,
    ) -> impl std::future::Future<Output = Result<(), J1939UnitError>> + Send {
        std::future::ready(Ok(()))
    }

    /// Teardown the unit.
    ///
    /// This method will be called to teardown the unit. This method should be non-blocking and should
    /// only perform asynchronous I/O operations. This method is optional and may be a no-op.
    fn teardown(
        &self,
        _ctx: &mut NetDriverContext,
        _network: &crate::net::ControlNetwork,
    ) -> impl std::future::Future<Output = Result<(), J1939UnitError>> + Send {
        std::future::ready(Ok(()))
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
    fn try_accept(
        &mut self,
        ctx: &mut NetDriverContext,
        network: &crate::net::ControlNetwork,
        signal_tx: crate::runtime::SignalSender,
    ) -> impl std::future::Future<Output = Result<(), J1939UnitError>> + Send;

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
