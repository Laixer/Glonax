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

pub type SharedRuntimeState = std::sync::Arc<tokio::sync::RwLock<RuntimeState>>;

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
    pub const DEFAULT_NETWORK_PORT: u16 = 30_051;
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

// TODO: Move somewhere else
pub struct EcuState {
    pub power: [std::sync::atomic::AtomicI16; 8],
    motion_lock: std::sync::atomic::AtomicBool,
}

impl EcuState {
    pub fn lock(&self) {
        self.power[0].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[1].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[2].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[3].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[4].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[5].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[6].store(0, std::sync::atomic::Ordering::Relaxed);
        self.power[7].store(0, std::sync::atomic::Ordering::Relaxed);

        self.motion_lock
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn unlock(&self) {
        self.motion_lock
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_locked(&self) -> bool {
        self.motion_lock.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for EcuState {
    fn default() -> Self {
        Self {
            power: [0; 8].map(|_| std::sync::atomic::AtomicI16::new(0)),
            motion_lock: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

// TODO: Rename to device state
#[derive(Default)]
pub struct RobotState {
    /// VMS data.
    pub vms: core::Host,
    /// GNSS data.
    pub gnss: core::Gnss,
    /// Engine data.
    pub engine: core::Engine,
    /// Pose data.
    pub pose: core::Pose,
}

// TODO: Rename to runtime
pub struct RuntimeState {
    /// Current machine state.
    pub status: core::Status,
    /// Glonax instance.
    pub instance: core::Instance,
    /// Robot state.
    pub state: RobotState,
    /// ECU state.
    pub ecu_state: EcuState,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            status: core::Status::Healthy,
            instance: core::Instance {
                id: "".to_string(), // TODO: Generate UUID
                model: "".to_string(),
                name: "".to_string(),
            },
            state: RobotState::default(),
            ecu_state: EcuState::default(),
        }
    }
}
