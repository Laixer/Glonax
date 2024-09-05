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
// pub mod components;
pub mod core;
pub mod driver;
pub mod logger;
pub mod math;
pub mod net;
pub mod protocol;
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

static INSTANCE: std::sync::OnceLock<core::Instance> = std::sync::OnceLock::new();

pub mod global {
    #[inline]
    pub fn instance() -> &'static crate::core::Instance {
        crate::INSTANCE.get().unwrap()
    }

    #[inline]
    pub fn set_instance(instance: crate::core::Instance) {
        crate::INSTANCE.set(instance).unwrap();
    }
}

/// Glonax runtime module containing various constants.
pub mod consts {
    use std::time::Duration;

    /// Glonax runtime version.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::VERSION;
    ///
    /// println!("Glonax runtime version: {}", VERSION);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the version of the Glonax runtime.
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    /// Glonax runtime major version.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::VERSION_MAJOR;
    ///
    /// println!("Glonax runtime major version: {}", VERSION_MAJOR);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the major version of the Glonax runtime.
    pub const VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");

    /// Glonax runtime minor version.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::VERSION_MINOR;
    ///
    /// println!("Glonax runtime minor version: {}", VERSION_MINOR);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the minor version of the Glonax runtime.
    pub const VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");

    /// Glonax runtime patch version.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::VERSION_PATCH;
    ///
    /// println!("Glonax runtime patch version: {}", VERSION_PATCH);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the patch version of the Glonax runtime.
    pub const VERSION_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");

    /// Glonax default network port for both TCP.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::DEFAULT_NETWORK_PORT;
    ///
    /// println!("Glonax default network port: {}", DEFAULT_NETWORK_PORT);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the default network port for both TCP in the Glonax runtime.
    pub const DEFAULT_NETWORK_PORT: u16 = 30_051;

    /// Glonax default queue size for commands.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::QUEUE_SIZE_COMMAND;
    ///
    /// println!("Glonax default queue size for commands: {}", QUEUE_SIZE_COMMAND);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the default queue size for commands in the Glonax runtime.
    pub const QUEUE_SIZE_COMMAND: usize = 16;

    /// Glonax default queue size for signals.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::QUEUE_SIZE_SIGNAL;
    ///
    /// println!("Glonax default queue size for signals: {}", QUEUE_SIZE_SIGNAL);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the default queue size for signals in the Glonax runtime.
    pub const QUEUE_SIZE_SIGNAL: usize = 16;

    /// Glonax network maximum number of clients.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::NETWORK_MAX_CLIENTS;
    ///
    /// println!("Glonax network maximum number of clients: {}", NETWORK_MAX_CLIENTS);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the maximum number of clients in the Glonax network.
    pub const NETWORK_MAX_CLIENTS: usize = 16;

    /// Glonax component delay threshold.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::COMPONENT_DELAY_THRESHOLD;
    /// use std::time::Duration;
    ///
    /// println!("Glonax component delay threshold: {:?}", COMPONENT_DELAY_THRESHOLD);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the delay threshold for Glonax components.
    pub const COMPONENT_DELAY_THRESHOLD: Duration = Duration::from_millis(1);

    /// Glonax service pipeline interval.
    ///
    /// # Example
    ///
    /// ```
    /// use glonax::consts::SERVICE_PIPELINE_INTERVAL;
    /// use std::time::Duration;
    ///
    /// println!("Glonax service pipeline interval: {:?}", SERVICE_PIPELINE_INTERVAL);
    /// ```
    ///
    /// # Remarks
    ///
    /// This constant represents the interval for the Glonax service pipeline.
    pub const SERVICE_PIPELINE_INTERVAL: Duration = Duration::from_millis(10);
}

/// Log system information.
///
/// This function logs system information including the system name, kernel version,
/// OS version, and host name.
pub fn log_system() {
    use sysinfo::System;

    log::debug!("System name: {}", System::name().unwrap_or_default());
    log::debug!(
        "System kernel version: {}",
        System::kernel_version().unwrap_or_default()
    );
    log::debug!(
        "System OS version: {}",
        System::os_version().unwrap_or_default()
    );
    log::debug!(
        "System architecture: {}",
        System::cpu_arch().unwrap_or_default()
    );
    log::debug!(
        "System host name: {}",
        System::host_name().unwrap_or_default()
    );
}

/// Check if the runtime version is compatible.
pub fn is_compatibile(version: (u8, u8, u8)) -> bool {
    let (major, minor, _) = version;

    major == consts::VERSION_MAJOR.parse().unwrap_or(0)
        && minor == consts::VERSION_MINOR.parse().unwrap_or(0)
}
