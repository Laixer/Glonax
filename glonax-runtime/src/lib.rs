// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

pub mod channel;
pub mod core;
pub mod device;
pub mod net;
pub mod robot;
pub mod transport;

#[macro_use]
extern crate log;

mod config;
pub use runtime::operand::Operand;

pub use self::config::*;

pub mod geometry;
pub mod telemetry;

pub mod runtime;
pub use self::runtime::builder::Builder as RuntimeBuilder;
pub use self::runtime::Error;
pub use self::runtime::RuntimeContext;

// TODO: Rename to consts
pub mod constants {
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
}
