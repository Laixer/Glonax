pub mod bosch_ems;
pub mod encoder;
pub mod engine;
pub mod fuzzer;
pub mod hydraulic;
pub mod inclino;
pub mod inspector;
pub mod vcu;
pub(super) mod vecraft;
pub mod vms;
pub mod volvo_ems;
mod volvo_vecu;

pub enum NetDriver {
    KueblerEncoder(super::KueblerEncoder),
    KueblerInclinometer(super::KueblerInclinometer),
    VolvoD7E(super::VolvoD7E),
    BoschEngineManagementSystem(super::BoschEngineManagementSystem),
    HydraulicControlUnit(super::HydraulicControlUnit),
    VehicleManagementSystem(super::VehicleManagementSystem),
    VehicleControlUnit(super::VehicleControlUnit),
}

impl NetDriver {
    pub(crate) fn factory(driver_type: &str, destination: u8, source: u8) -> Result<Self, ()> {
        match driver_type {
            "kuebler_encoder" => Ok(NetDriver::KueblerEncoder(
                crate::driver::KueblerEncoder::new(destination, source),
            )),
            "kuebler_inclinometer" => Ok(NetDriver::KueblerInclinometer(
                crate::driver::KueblerInclinometer::new(destination, source),
            )),
            "volvo_d7e" => Ok(NetDriver::VolvoD7E(crate::driver::VolvoD7E::new(
                destination,
                source,
            ))),
            "bosch_ems" => Ok(NetDriver::BoschEngineManagementSystem(
                crate::driver::BoschEngineManagementSystem::new(destination, source),
            )),
            "hydraulic_control_unit" => Ok(NetDriver::HydraulicControlUnit(
                crate::driver::HydraulicControlUnit::new(destination, source),
            )),
            "vehicle_control_unit" => Ok(NetDriver::VehicleControlUnit(
                crate::driver::VehicleControlUnit::new(destination, source),
            )),
            _ => Err(()),
        }
    }
}

impl J1939Unit for NetDriver {
    fn name(&self) -> &str {
        match self {
            Self::KueblerEncoder(encoder) => encoder.name(),
            Self::KueblerInclinometer(inclinometer) => inclinometer.name(),
            Self::VolvoD7E(volvo) => volvo.name(),
            Self::BoschEngineManagementSystem(bosch) => bosch.name(),
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
            Self::BoschEngineManagementSystem(bosch) => bosch.destination(),
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
            Self::BoschEngineManagementSystem(bosch) => bosch.source(),
            Self::HydraulicControlUnit(hydraulic) => hydraulic.source(),
            Self::VehicleManagementSystem(responder) => responder.source(),
            Self::VehicleControlUnit(vcu) => vcu.source(),
        }
    }

    async fn setup(
        &self,
        ctx: &mut NetDriverContext,
        router: &crate::net::ControlNetwork,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => encoder.setup(ctx, router, runtime_state).await,
            Self::KueblerInclinometer(inclinometer) => {
                inclinometer.setup(ctx, router, runtime_state).await
            }
            Self::VolvoD7E(volvo) => volvo.setup(ctx, router, runtime_state).await,
            Self::BoschEngineManagementSystem(bosch) => {
                bosch.setup(ctx, router, runtime_state).await
            }
            Self::HydraulicControlUnit(hydraulic) => {
                hydraulic.setup(ctx, router, runtime_state).await
            }
            Self::VehicleManagementSystem(responder) => {
                responder.setup(ctx, router, runtime_state).await
            }
            Self::VehicleControlUnit(vcu) => vcu.setup(ctx, router, runtime_state).await,
        }
    }

    async fn teardown(
        &self,
        ctx: &mut NetDriverContext,
        network: &crate::net::ControlNetwork,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => encoder.teardown(ctx, network, runtime_state).await,
            Self::KueblerInclinometer(inclinometer) => {
                inclinometer.teardown(ctx, network, runtime_state).await
            }
            Self::VolvoD7E(volvo) => volvo.teardown(ctx, network, runtime_state).await,
            Self::BoschEngineManagementSystem(bosch) => {
                bosch.teardown(ctx, network, runtime_state).await
            }
            Self::HydraulicControlUnit(hydraulic) => {
                hydraulic.teardown(ctx, network, runtime_state).await
            }
            Self::VehicleManagementSystem(responder) => {
                responder.teardown(ctx, network, runtime_state).await
            }
            Self::VehicleControlUnit(vcu) => vcu.teardown(ctx, network, runtime_state).await,
        }
    }

    async fn try_accept(
        &mut self,
        ctx: &mut NetDriverContext,
        network: &crate::net::ControlNetwork,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => encoder.try_accept(ctx, network, runtime_state).await,
            Self::KueblerInclinometer(inclinometer) => {
                inclinometer.try_accept(ctx, network, runtime_state).await
            }
            Self::VolvoD7E(volvo) => volvo.try_accept(ctx, network, runtime_state).await,
            Self::BoschEngineManagementSystem(bosch) => {
                bosch.try_accept(ctx, network, runtime_state).await
            }
            Self::HydraulicControlUnit(hydraulic) => {
                hydraulic.try_accept(ctx, network, runtime_state).await
            }
            Self::VehicleManagementSystem(responder) => {
                responder.try_accept(ctx, network, runtime_state).await
            }
            Self::VehicleControlUnit(vcu) => vcu.try_accept(ctx, network, runtime_state).await,
        }
    }

    async fn tick(
        &self,
        ctx: &mut NetDriverContext,
        network: &crate::net::ControlNetwork,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => encoder.tick(ctx, network, runtime_state).await,
            Self::KueblerInclinometer(inclinometer) => {
                inclinometer.tick(ctx, network, runtime_state).await
            }
            Self::VolvoD7E(volvo) => volvo.tick(ctx, network, runtime_state).await,
            Self::BoschEngineManagementSystem(bosch) => {
                bosch.tick(ctx, network, runtime_state).await
            }
            Self::HydraulicControlUnit(hydraulic) => {
                hydraulic.tick(ctx, network, runtime_state).await
            }
            Self::VehicleManagementSystem(responder) => {
                responder.tick(ctx, network, runtime_state).await
            }
            Self::VehicleControlUnit(vcu) => vcu.tick(ctx, network, runtime_state).await,
        }
    }

    async fn trigger(
        &self,
        ctx: &mut NetDriverContext,
        network: &crate::net::ControlNetwork,
        runtime_state: crate::runtime::SharedOperandState,
        trigger: &crate::core::Motion,
    ) -> Result<(), J1939UnitError> {
        match self {
            Self::KueblerEncoder(encoder) => {
                encoder.trigger(ctx, network, runtime_state, trigger).await
            }
            Self::KueblerInclinometer(inclinometer) => {
                inclinometer
                    .trigger(ctx, network, runtime_state, trigger)
                    .await
            }
            Self::VolvoD7E(volvo) => volvo.trigger(ctx, network, runtime_state, trigger).await,
            Self::BoschEngineManagementSystem(bosch) => {
                bosch.trigger(ctx, network, runtime_state, trigger).await
            }
            Self::HydraulicControlUnit(hydraulic) => {
                hydraulic.trigger(ctx, network, runtime_state, trigger).await
            }
            Self::VehicleManagementSystem(responder) => {
                responder.trigger(ctx, network, runtime_state, trigger).await
            }
            Self::VehicleControlUnit(vcu) => vcu.trigger(ctx, network, runtime_state, trigger).await,
        }
    }
}

pub struct NetDriverContext {
    /// Last time a message was sent.
    tx_last: std::time::Instant,
    /// Last time a message was received.
    rx_last: std::time::Instant,
}

impl NetDriverContext {
    fn is_rx_timeout(&self, timeout: std::time::Duration) -> bool {
        self.rx_last.elapsed() > timeout
    }

    fn tx_mark(&mut self) {
        self.tx_last = std::time::Instant::now();
    }

    fn rx_mark(&mut self) {
        self.rx_last = std::time::Instant::now();
    }
}

impl Default for NetDriverContext {
    fn default() -> Self {
        Self {
            tx_last: std::time::Instant::now(),
            rx_last: std::time::Instant::now(),
        }
    }
}

#[derive(Default)]
pub struct NetDriverCollection(Vec<(NetDriver, NetDriverContext)>);

impl NetDriverCollection {
    /// Register a driver with the control network.
    pub fn register_driver(&mut self, driver: NetDriver) {
        self.0.push((driver, NetDriverContext::default()));
    }

    pub fn inner_mut(&mut self) -> &mut Vec<(NetDriver, NetDriverContext)> {
        &mut self.0
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

// FUTURE: Maybe move to runtime or a network module?
pub trait J1939Unit {
    /// Get the name of the unit.
    fn name(&self) -> &str;

    /// Get the destination address of the unit.
    fn destination(&self) -> u8;

    /// Get the source address of the unit.
    fn source(&self) -> u8;

    fn setup(
        &self,
        _ctx: &mut NetDriverContext,
        _network: &crate::net::ControlNetwork,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> impl std::future::Future<Output = Result<(), J1939UnitError>> + Send {
        std::future::ready(Ok(()))
    }

    fn teardown(
        &self,
        _ctx: &mut NetDriverContext,
        _network: &crate::net::ControlNetwork,
        _runtime_state: crate::runtime::SharedOperandState,
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
        _network: &crate::net::ControlNetwork,
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
        _network: &crate::net::ControlNetwork,
        _runtime_state: crate::runtime::SharedOperandState,
        _trigger: &crate::core::Motion,
    ) -> impl std::future::Future<Output = Result<(), J1939UnitError>> + Send {
        std::future::ready(Ok(()))
    }
}
