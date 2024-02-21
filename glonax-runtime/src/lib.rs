// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

pub mod components;
pub mod core;
pub mod driver;
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
    pub const DEFAULT_SOCKET_PATH: &str = "/tmp/glonax.sock";
    /// Glonax default configuration path.
    pub const DEFAULT_CONFIG_PATH: &str = "/etc/glonax/glonax.toml";
    /// Glonax default queue size for motion commands.
    pub const QUEUE_SIZE_MOTION: usize = 32;
    /// Glonax network maximum number of clients.
    pub const NETWORK_MAX_CLIENTS: usize = 16;
}

// TODO: Return the machine state in its entirety over the network
// TODO: Integrate into the operand
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

/// The operand is the current state of the machine.
///
/// This is the state that is used by the runtime to control
/// the machine and the state that is used by the middleware.
pub struct Operand {
    /// Robot state.
    pub state: MachineState,
}

impl Operand {
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
