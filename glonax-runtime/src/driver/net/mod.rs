pub mod bosch_ems;
pub mod encoder;
pub mod engine;
pub mod fuzzer;
pub mod hydraulic;
pub mod inclino;
pub mod inspector;
pub mod reqres;
pub mod vcu;
pub(super) mod vecraft;
pub mod volvo_ems;
mod volvo_vecu;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetDriverConfig {
    /// Destination address.
    pub destination: u8,
    /// Source address.
    pub source: u8,
    /// Driver type.
    pub driver_type: String,
}

pub enum NetDriver {
    KueblerEncoder(super::KueblerEncoder),
    KueblerInclinometer(super::KueblerInclinometer),
    VolvoD7E(super::VolvoD7E),
    BoschEngineManagementSystem(super::BoschEngineManagementSystem),
    HydraulicControlUnit(super::HydraulicControlUnit),
    RequestResponder(super::RequestResponder),
    VehicleControlUnit(super::VehicleControlUnit),
}

impl NetDriver {
    pub fn request_responder(address: u8) -> crate::driver::net::NetDriver {
        crate::driver::net::NetDriver::RequestResponder(crate::driver::RequestResponder::new(
            address,
        ))
    }
}

impl TryFrom<NetDriverConfig> for NetDriver {
    // type Error = crate::Error;
    type Error = ();

    fn try_from(config: NetDriverConfig) -> Result<Self, Self::Error> {
        match config.driver_type.as_str() {
            "kuebler_encoder" => Ok(NetDriver::KueblerEncoder(
                crate::driver::KueblerEncoder::new(config.destination, config.source),
            )),
            "kuebler_inclinometer" => Ok(NetDriver::KueblerInclinometer(
                crate::driver::KueblerInclinometer::new(config.destination, config.source),
            )),
            "volvo_d7e" => Ok(NetDriver::VolvoD7E(crate::driver::VolvoD7E::new(
                config.destination,
                config.source,
            ))),
            "hydraulic_control_unit" => Ok(NetDriver::HydraulicControlUnit(
                crate::driver::HydraulicControlUnit::new(config.destination, config.source),
            )),
            "request_responder" => Ok(NetDriver::RequestResponder(
                crate::driver::RequestResponder::new(config.source),
            )),
            "vehicle_control_unit" => Ok(NetDriver::VehicleControlUnit(
                crate::driver::VehicleControlUnit::new(config.destination, config.source),
            )),
            _ => Err(()),
        }
    }
}

pub struct NetDriverContext {
    /// Last time a message was sent.
    pub tx_last: std::time::Instant,
    /// Last time a message was received.
    pub rx_last: std::time::Instant,
}

#[derive(Debug)]
pub enum J1939UnitError {
    /// Unit has not sent a message in a while.
    MessageTimeout,
    /// Unit has an invalid configuration.
    InvalidConfiguration,
    /// Version mismatch.
    VersionMismatch,
    /// Bus error.
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

// #[derive(Debug)]
// pub struct J1939UnitError {
//     name: String,
//     destination: u8,
//     kind: J1939UnitErrorKind,
// }

// impl J1939UnitError {
//     /// Construct a new error.
//     pub fn new(name: String, destination: u8, kind: J1939UnitErrorKind) -> Self {
//         Self {
//             name,
//             destination,
//             kind,
//         }
//     }

//     /// Get the name of the unit.
//     pub fn name(&self) -> &str {
//         &self.name
//     }

//     /// Get the destination address of the unit.
//     pub fn destination(&self) -> u8 {
//         self.destination
//     }

//     /// Get the kind of the error.
//     pub fn kind(&self) -> &J1939UnitErrorKind {
//         &self.kind
//     }
// }

// impl std::fmt::Display for J1939UnitError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(
//             f,
//             "{} (destination: {}): {}",
//             self.name, self.destination, self.kind
//         )
//     }
// }

// impl std::error::Error for J1939UnitError {}

/// Operational states for a J1939 unit.
///
/// A unit can transition between these states during its lifetime,
/// however, the order of the states is fixed. The unit will always
/// start in the `Setup` state and end in the `Teardown` state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum J1939UnitOperationState {
    /// The unit is in the setup phase.
    Setup,
    /// The unit is running.
    Running,
    /// The unit is in the teardown phase.
    Teardown,
}

// FUTURE: Maybe move to runtime or a network module?
pub trait J1939Unit {
    /// Get the name of the unit.
    fn name(&self) -> &str;

    /// Get the destination address of the unit.
    fn destination(&self) -> u8;

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
        state: &J1939UnitOperationState,
        router: &crate::net::Router,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> impl std::future::Future<Output = Result<(), J1939UnitError>> + Send;

    /// Tick the unit on interval.
    ///
    /// This method will be called on interval to allow the unit to perform any necessary
    /// operations. This method should be non-blocking and should only perform asynchronous
    /// I/O operations.
    ///
    /// This method is optional and may be a no-op.
    fn tick(
        &self,
        _ctx: &mut NetDriverContext,
        _state: &J1939UnitOperationState,
        _router: &crate::net::Router,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> impl std::future::Future<Output = Result<(), J1939UnitError>> + Send {
        std::future::ready(Ok(()))
    }

    /// Trigger the unit manually.
    ///
    /// This method will be called to trigger the unit manually. This method should be non-blocking
    /// and should only perform asynchronous I/O operations.
    ///
    /// This method is optional and may be a no-op.
    fn trigger(
        &self,
        _ctx: &mut NetDriverContext,
        _state: &J1939UnitOperationState,
        _router: &crate::net::Router,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> impl std::future::Future<Output = Result<(), J1939UnitError>> + Send {
        std::future::ready(Ok(()))
    }
}
