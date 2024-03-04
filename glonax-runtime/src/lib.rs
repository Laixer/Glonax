// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

/// The `glonax-runtime` library provides a runtime environment for the Glonax system.
///
/// This library contains modules for core functionality, drivers, logging, mathematics,
/// networking, protocols, robots, services, and the world. It also includes a `can` module
/// for CAN bus communication. The library exports the `config` module and re-exports the
/// `j1939` and `rand` crates.
///
/// The `runtime` module provides the `Runtime` struct and the `Error` enum for managing
/// the Glonax runtime. The `consts` module defines various constants used in the runtime,
/// such as the version, default network port, default socket path, default configuration path,
/// queue size for motion commands, and maximum number of network clients.
///
/// The `MachineState` struct represents the state of a machine in the Glonax system. It
/// includes data for the vehicle management system, global navigation satellite system,
/// engine, engine request, hydraulic quick disconnect, hydraulic lock, motion, encoders,
/// robot actor, program queue, and electronic control unit state.
///
/// The `Governor` struct represents a governor for the engine. It has fields for the default
/// engine speed and maximum RPM. It provides methods for reshaping torque and determining
/// the engine mode based on the actual and requested engine modes.
///
/// The `Operand` struct represents the operand, which is the current state of the machine.
/// It includes the machine state and a governor for the engine. It provides methods for
/// determining the governor mode and the status of the machine.
pub mod core;
pub mod driver;
pub mod logger;
pub mod math;
pub mod net;
pub mod protocol;
pub mod robot;
pub mod service;
pub mod world;

mod can;

#[macro_use]
extern crate log;

mod config;

pub use self::config::*;

pub use j1939;
pub use rand;

pub mod runtime;
pub use self::runtime::Error;
pub use self::runtime::Runtime;

pub mod consts {
    /// Glonax runtime version.
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    /// Glonax runtime major version.
    pub const VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
    /// Glonax runtime minor version.
    pub const VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
    /// Glonax runtime patch version.
    pub const VERSION_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");
    /// Glonax default network port for both TCP.
    pub const DEFAULT_NETWORK_PORT: u16 = 30_051;
    /// Glonax default unix socket path.
    pub const DEFAULT_SOCKET_PATH: &str = "/run/glonax/glonax.sock"; // TODO: get from env $RUNTIME_DIRECTORY
    /// Glonax default configuration path.
    pub const DEFAULT_CONFIG_PATH: &str = "/etc/glonax/glonax.toml"; // TODO: get from env $CONFIGURATION_DIRECTORY
    /// Glonax default queue size for motion commands.
    pub const QUEUE_SIZE_MOTION: usize = 32;
    /// Glonax network maximum number of clients.
    pub const NETWORK_MAX_CLIENTS: usize = 16;
}

// TODO: Return the machine state in its entirety over the network
// TODO: Integrate into the operand
/// Represents the state of a machine.
///
/// The project refers to the machine as the entire system including
/// hardware, software, sensors and actuators.
#[derive(Default)]
pub struct MachineState {
    /// Vehicle management system data.
    pub vms: core::Host,
    /// Global navigation satellite system data.
    pub gnss: core::Gnss,
    /// Engine data.
    pub engine: core::Engine,
    /// Engine state actual.
    pub engine_state_actual: core::EngineState,
    /// Engine state request.
    pub engine_state_request: core::EngineState,
    /// Engine requested RPM.
    pub engine_request: u16, // TODO: Move into engine request struct
    /// Hydraulic quick disconnect.
    pub hydraulic_quick_disconnect: bool, // TODO: Move into hydraulic request struct
    /// Hydraulic lock.
    pub hydraulic_lock: bool, // TODO: Move into hydraulic request struct
    /// Motion data.
    pub motion: core::Motion,
    /// Encoder data.
    pub encoders: std::collections::HashMap<u8, f32>, // TODO: Remove from here
    /// Robot as an actor.
    pub actor: Option<crate::world::Actor>, // TODO: Remove from here
    /// Current program queue.
    pub program: std::collections::VecDeque<core::Target>,
    /// Electronic control unit data.
    pub ecu_state: driver::VirtualHCU,
}

struct Governor {
    /// Default engine speed.
    rpm_idle: u16,
    /// Maximum RPM for the engine.
    rpm_max: u16,
}

impl Governor {
    /// Construct a new governor.
    fn new(rpm_idle: u16, rpm_max: u16) -> Self {
        Self { rpm_idle, rpm_max }
    }

    fn reshape(&self, torque: u16) -> u16 {
        torque.clamp(self.rpm_idle, self.rpm_max)
    }

    /// Determine the next engine state based on the actual and requested states.
    ///
    /// # Arguments
    ///
    /// * `actual` - The actual engine mode.
    /// * `request` - The requested engine mode.
    ///
    /// # Returns
    ///
    /// The resulting engine mode.
    fn mode(
        &self,
        actual: &core::EngineState,
        request: &core::EngineState,
    ) -> crate::core::EngineState {
        use crate::core::EngineState;

        match (actual, request) {
            (EngineState::NoRequest, EngineState::Starting(_)) => {
                EngineState::Starting(self.rpm_idle)
            }
            (EngineState::NoRequest, EngineState::Request(_)) => {
                EngineState::Starting(self.rpm_idle)
            }
            (EngineState::NoRequest, _) => EngineState::NoRequest,
            (EngineState::Starting(_), _) => EngineState::Starting(self.rpm_idle),
            (EngineState::Stopping, _) => EngineState::Stopping,
            (EngineState::Request(_), EngineState::NoRequest) => EngineState::Stopping,
            (EngineState::Request(r), EngineState::Starting(_)) => {
                EngineState::Request(self.reshape(*r))
            }
            (EngineState::Request(_), EngineState::Stopping) => EngineState::Stopping,
            (EngineState::Request(_), EngineState::Request(r)) => {
                EngineState::Request(self.reshape(*r))
            }
        }
    }
}

/// The operand is the current state of the machine.
///
/// This is the state that is used by the runtime to control
/// the machine and the state that is used by the middleware.
pub struct Operand {
    /// Current machine state.
    pub state: MachineState,
    /// Governor for the engine.
    governor: Governor,
}

impl Operand {
    /// Get the governor mode for the engine.
    ///
    /// This method determines the governor mode based on the current
    /// engine request and the actual engine mode.
    pub fn governor_mode(&self) -> crate::core::EngineState {
        let request = if self.state.engine_request == 0 {
            core::EngineState::NoRequest
        } else {
            core::EngineState::Request(self.state.engine_request)
        };

        self.governor
            .mode(&self.state.engine_state_actual, &request)
    }

    /// Get the status of the machine.
    ///
    /// This method returns the status of the machine based on the
    /// current machine state. It takes into account the status of
    /// the vehicle management system, global navigation satellite
    /// system, engine, and other factors.
    pub fn status(&self) -> core::Status {
        use crate::core::{EngineStatus, GnssStatus, HostStatus, Status};

        let mut status = Status::Healthy;

        match self.state.vms.status {
            HostStatus::MemoryLow => {
                status = Status::Degraded;
            }
            HostStatus::CPUHigh => {
                status = Status::Degraded;
            }
            _ => {}
        }

        if let GnssStatus::DeviceNotFound = self.state.gnss.status {
            status = Status::Faulty;
        }

        match self.state.engine.status {
            EngineStatus::NetworkDown => {
                status = Status::Faulty;
            }
            EngineStatus::MessageTimeout => {
                status = Status::Degraded;
            }
            _ => {}
        }

        status
    }
}
