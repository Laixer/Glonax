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
pub mod components;
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

/// Represents the state of a machine.
///
/// The project refers to the machine as the entire system including
/// hardware, software, sensors and actuators.
#[derive(Default)]
pub struct MachineState {
    /// Vehicle management system data.
    pub vms_signal: core::Host, // SIGNAL
    /// Vehicle management system update.
    pub vms_signal_instant: Option<std::time::Instant>,

    /// Engine signal.
    pub engine_signal: core::Engine, // SIGNAL
    /// Engine state actual instant.
    pub engine_signal_instant: Option<std::time::Instant>,

    /// Motion locked.
    pub motion_locked: bool, // SIGNAL
    /// Motion data.
    pub motion_command: core::Motion, // INNER SERVICE (hydraulic)
    /// Motion instant.
    pub motion_command_instant: Option<std::time::Instant>,

    /// Encoder data.
    pub encoders: std::collections::HashMap<u8, f32>, // TODO: Remove from here // SIGNAL
    /// Encoder instant.
    pub encoders_instant: Option<std::time::Instant>,

    /// Electronic control unit data.
    pub ecu_state: driver::VirtualHCU, // CROSS SERVICE (Sim actuator, sim encoder)
}

/// The operand is the current state of the machine.
///
/// This is the state that is used by the runtime to control
/// the machine and the state that is used by the middleware.
pub struct Operand {
    /// Current machine state.
    pub state: MachineState,
}

impl Operand {
    // TODO: Report all statuses, not just a single one
    /// Get the status of the machine.
    ///
    /// This method returns the status of the machine based on the current machine state. It takes
    /// into account the status of the vehicle management system, global navigation satellite system,
    /// engine, and other factors.
    pub fn status(&self) -> core::Status {
        use crate::core::{HostStatus, Status};

        let mut status = Status::Healthy;

        match self.state.vms_signal.status {
            HostStatus::MemoryLow => {
                status = Status::Degraded;
            }
            HostStatus::CPUHigh => {
                status = Status::Degraded;
            }
            _ => {}
        }

        // if let GnssStatus::DeviceNotFound = self.state.gnss_signal.status {
        //     status = Status::Faulty;
        // }

        // match self.state.engine.status {
        //     EngineStatus::NetworkDown => {
        //         status = Status::Faulty;
        //     }
        //     EngineStatus::MessageTimeout => {
        //         status = Status::Degraded;
        //     }
        //     _ => {}
        // }

        status
    }
}
