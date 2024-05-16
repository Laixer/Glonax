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

// TODO: Return the machine state in its entirety over the network
// TODO: Integrate into the operand
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

    /// Global navigation satellite system data.
    pub gnss_signal: core::Gnss, // SIGNAL
    /// GNSS signal update.
    pub gnss_signal_instant: Option<std::time::Instant>,

    /// Engine signal.
    pub engine_signal: core::Engine, // SIGNAL
    /// Engine state actual instant.
    pub engine_signal_instant: Option<std::time::Instant>,
    /// Engine command.
    pub engine_command: Option<core::Engine>, // INNER SERVICE (engine)
    /// Engine state request instant.
    pub engine_command_instant: Option<std::time::Instant>,

    /// Hydraulic quick disconnect.
    pub hydraulic_quick_disconnect: bool, // TODO: Move into hydraulic request struct // COMMAND
    /// Hydraulic lock.
    pub hydraulic_lock: bool, // TODO: Move into hydraulic request struct // COMMAND
    /// Hydraulic actual instant.
    pub hydraulic_actual_instant: Option<std::time::Instant>,

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

    /// Robot as an actor.
    pub actor: Option<crate::world::Actor>, // TODO: Remove from here // SIGNAL
    /// Robot as an actor instant.
    pub actor_instant: Option<std::time::Instant>,

    /// Current program queue.
    pub program: std::collections::VecDeque<core::Target>, // TODO: Move into component state? // COMMAND
    /// Electronic control unit data.
    pub ecu_state: driver::VirtualHCU, // CROSS SERVICE (Sim actuator, sim encoder)
}

struct Governor {
    /// Default engine speed.
    rpm_idle: u16,
    /// Maximum RPM for the engine.
    rpm_max: u16,
    /// Engine state transition timeout.
    state_transition_timeout: std::time::Duration,
}

impl Governor {
    /// Construct a new governor.
    fn new(rpm_idle: u16, rpm_max: u16) -> Self {
        Self {
            rpm_idle,
            rpm_max,
            state_transition_timeout: std::time::Duration::from_millis(2_000),
        }
    }

    /// Reshape the torque.
    ///
    /// This method reshapes the torque based on the engine speed.
    #[inline]
    fn reshape(&self, torque: u16) -> u16 {
        torque.clamp(self.rpm_idle, self.rpm_max)
    }

    /// Get the next engine state.
    ///
    /// This method determines the next engine state based on the actual and requested
    /// engine states. It returns the next engine state as an `EngineRequest`.
    fn next_state(
        &self,
        signal: &core::Engine,
        command: &core::Engine,
        command_instant: Option<std::time::Instant>,
    ) -> crate::core::Engine {
        use crate::core::EngineState;

        match (signal.state, command.state) {
            (EngineState::NoRequest, EngineState::Starting) => {
                if let Some(instant) = command_instant {
                    if instant.elapsed() > self.state_transition_timeout {
                        return core::Engine {
                            rpm: self.reshape(self.rpm_idle),
                            state: EngineState::NoRequest,
                            ..Default::default()
                        };
                    }
                }

                core::Engine {
                    rpm: self.reshape(self.rpm_idle),
                    state: EngineState::Starting,
                    ..Default::default()
                }
            }
            (EngineState::NoRequest, EngineState::Request) => {
                if let Some(instant) = command_instant {
                    if instant.elapsed() > self.state_transition_timeout {
                        return core::Engine {
                            rpm: self.reshape(self.rpm_idle),
                            state: EngineState::NoRequest,
                            ..Default::default()
                        };
                    }
                }

                core::Engine {
                    rpm: self.reshape(self.rpm_idle),
                    state: EngineState::Starting,
                    ..Default::default()
                }
            }
            (EngineState::NoRequest, _) => core::Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::NoRequest,
                ..Default::default()
            },

            (EngineState::Starting, _) => {
                if let Some(instant) = command_instant {
                    if instant.elapsed() > self.state_transition_timeout {
                        return core::Engine {
                            rpm: self.reshape(self.rpm_idle),
                            state: EngineState::NoRequest,
                            ..Default::default()
                        };
                    }
                }

                core::Engine {
                    rpm: self.reshape(self.rpm_idle),
                    state: EngineState::Starting,
                    ..Default::default()
                }
            }
            (EngineState::Stopping, _) => core::Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::Stopping,
                ..Default::default()
            },

            (EngineState::Request, EngineState::NoRequest) => core::Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::Stopping,
                ..Default::default()
            },
            (EngineState::Request, EngineState::Starting) => core::Engine {
                rpm: self.reshape(command.rpm),
                state: EngineState::Request,
                ..Default::default()
            },
            (EngineState::Request, EngineState::Stopping) => core::Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::Stopping,
                ..Default::default()
            },
            (EngineState::Request, EngineState::Request) => core::Engine {
                rpm: self.reshape(command.rpm),
                state: EngineState::Request,
                ..Default::default()
            },
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
    /// engine request, the actual engine mode and the last motion update.
    ///
    /// If no engine state request is present, the governor will use the
    /// actual engine state, essentially maintaining the current state.
    pub fn governor_mode(&self) -> crate::core::Engine {
        let mut engine_command = self
            .state
            .engine_command
            .unwrap_or(self.state.engine_signal);
        engine_command.actual_engine = 0;
        engine_command.state = match engine_command.state {
            core::EngineState::NoRequest => core::EngineState::NoRequest,
            core::EngineState::Request => core::EngineState::Request,
            _ => self.state.engine_signal.state,
        };

        engine_command.driver_demand = engine_command.driver_demand.clamp(0, 100);

        if engine_command.rpm == 0 {
            if engine_command.driver_demand == 0 {
                engine_command.state = core::EngineState::NoRequest;
            } else {
                engine_command.rpm = (engine_command.driver_demand as f32 / 100.0
                    * self.governor.rpm_max as f32) as u16;
            }
        } else {
            engine_command.state = core::EngineState::Request;
        }

        let engine_state = self.governor.next_state(
            &self.state.engine_signal,
            &engine_command,
            self.state.engine_command_instant,
        );

        log::trace!("Engine governor: {:?}", engine_state);

        engine_state
    }

    // TODO: Report all statuses, not just a single one
    /// Get the status of the machine.
    ///
    /// This method returns the status of the machine based on the current machine state. It takes
    /// into account the status of the vehicle management system, global navigation satellite system,
    /// engine, and other factors.
    pub fn status(&self) -> core::Status {
        use crate::core::{GnssStatus, HostStatus, Status};

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

        if let GnssStatus::DeviceNotFound = self.state.gnss_signal.status {
            status = Status::Faulty;
        }

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
