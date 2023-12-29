// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

pub mod channel;
pub mod components;
pub mod core;
pub mod device;
pub mod math;
pub mod net;
pub mod protocol;

#[macro_use]
extern crate log;

#[macro_use]
extern crate static_assertions;

mod config;

pub use self::config::*;

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
    /// Glonax default J1939 address.
    pub const DEFAULT_J1939_ADDRESS: u8 = 0x9E;
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

// TODO: Integrate into the operand
#[derive(Default)]
pub struct MachineState {
    /// Vehicle management system data.
    pub vms: core::Host,
    /// Global navigation satellite system data.
    pub gnss: core::Gnss,
    /// Engine data.
    pub engine: core::Engine,
    /// Pose data.
    pub pose: core::Pose,
    /// Encoder data.
    pub encoders: nalgebra::Rotation3<f32>,
    /// Target position.
    pub target: Option<core::Target>,
    /// Electronic control unit data.
    pub ecu_state: device::VirtualHCU,
}

/// The operand is the current state of the machine.
///
/// This is the state that is used by the runtime to control
/// the machine and the state that is used by the middleware.
pub struct Operand {
    /// Current machine state.
    pub status: core::Status,
    /// Glonax instance.
    pub instance: core::Instance,
    /// Robot state.
    pub state: MachineState,
}

impl Default for Operand {
    fn default() -> Self {
        Self {
            status: core::Status::Healthy,
            instance: core::Instance::default(),
            state: MachineState::default(),
        }
    }
}
