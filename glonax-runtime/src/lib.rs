// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

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
    rpm_start: u16,
    rpm_idle: u16,
    rpm_max: u16,
}

impl Governor {
    fn new(rpm_start: u16, rpm_idle: u16, rpm_max: u16) -> Self {
        Self {
            rpm_start,
            rpm_idle,
            rpm_max,
        }
    }

    fn mode(&self, engine: &core::Engine, engine_request: u16) -> crate::core::EngineMode {
        let engine_request = engine_request.clamp(self.rpm_idle, self.rpm_max);

        if engine_request == 0 {
            crate::core::EngineMode::NoRequest
        } else if engine.rpm == 0 || engine.rpm < self.rpm_start {
            crate::core::EngineMode::Start
        } else {
            crate::core::EngineMode::Request(engine_request)
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
    pub fn governor(&self) -> crate::core::EngineMode {
        // const ENGINE_RPM_START: u16 = 500;
        // const ENGINE_RPM_IDLE: u16 = 700;
        // const ENGINE_RPM_MAX: u16 = 2_100;

        // let engine = self.state.engine;
        // let engine_request = self
        //     .state
        //     .engine_request
        //     .clamp(ENGINE_RPM_IDLE, ENGINE_RPM_MAX);

        // // TODO: Missing off=off
        // if engine_request == 0 {
        //     crate::core::EngineMode::NoRequest
        // } else if engine.rpm == 0 || engine.rpm < ENGINE_RPM_START {
        //     crate::core::EngineMode::Start
        // } else {
        //     crate::core::EngineMode::Request(engine_request)
        // }
        self.governor
            .mode(&self.state.engine, self.state.engine_request)
    }

    /// Current machine state.
    ///
    /// This method returns the current machine state based
    /// on the current operand state. It is a convenience
    /// method to avoid having to lock the operand state
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
