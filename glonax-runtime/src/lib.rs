// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

pub mod channel;
pub mod core;
pub mod device;
pub mod net;
pub mod transport;

#[macro_use]
extern crate log;

mod config;
pub use runtime::operand::Operand;

pub use self::config::*;

pub mod runtime;
pub use self::runtime::builder::Builder as RuntimeBuilder;
pub use self::runtime::Error;
pub use self::runtime::RuntimeContext;

use crate::core::{Engine, Gnss, Host, Pose};

pub struct RobotState {
    /// VMS telemetry data.
    pub vms: Host,
    /// GNSS telemetry data.
    pub gnss: Gnss,
    /// Engine telemetry data.
    pub engine: Engine,
    /// Pose telemetry data.
    pub pose: Pose,
}

impl Default for RobotState {
    fn default() -> Self {
        Self {
            vms: Host::default(),
            gnss: Gnss::default(),
            engine: Engine::default(),
            pose: Pose::default(),
        }
    }
}

// TODO: Rename to runtime session
pub struct MachineState {
    /// Current machine state.
    pub status: core::Status,
    /// Glonax instance.
    pub instance: core::Instance,
    /// Robot state.
    pub state: RobotState,
}

impl MachineState {
    pub fn new() -> Self {
        Self {
            status: core::Status::Healthy,
            instance: core::Instance {
                id: "".to_string(), // TODO: Generate UUID
                model: "".to_string(),
                name: "".to_string(),
            },
            state: RobotState::default(),
        }
    }
}

pub mod consts {
    /// Glonax runtime version.
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    /// Glonax runtime major version.
    pub const VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
    /// Glonax runtime minor version.
    pub const VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
    /// Glonax runtime patch version.
    pub const VERSION_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");
    /// Glonax default J1939 address.
    pub const DEFAULT_J1939_ADDRESS: u8 = 0x9E;
    /// Glonax default network port for both TCP and UDP.
    pub const DEFAULT_NETWORK_PORT: u16 = 30051;
    /// Glonax default configuration path.
    pub const DEFAULT_CONFIG_PATH: &str = "/etc/glonax/glonax.toml";
    /// Signal FIFO file located in the working directory.
    pub const FIFO_SIGNAL_FILE: &str = "signal";
    /// Glonax default queue size for signals.
    pub const QUEUE_SIZE_SIGNAL: usize = 32;
    /// Glonax default queue size for motion commands.
    pub const QUEUE_SIZE_MOTION: usize = 32;
    /// Glonax network maximum number of clients.
    pub const NETWORK_MAX_CLIENTS: usize = 16;
}
