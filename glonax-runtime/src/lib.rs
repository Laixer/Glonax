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

pub use self::config::*;

pub mod runtime;
pub use self::runtime::builder::Builder as RuntimeBuilder;
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
    /// Glonax default network port for both TCP and UDP.
    pub const DEFAULT_NETWORK_PORT: u16 = 30_051;
    /// Glonax default configuration path.
    pub const DEFAULT_CONFIG_PATH: &str = "/etc/glonax/glonax.toml";
    /// Glonax default queue size for motion commands.
    pub const QUEUE_SIZE_MOTION: usize = 32;
    /// Glonax network maximum number of clients.
    pub const NETWORK_MAX_CLIENTS: usize = 16;
}

use std::sync::atomic::{AtomicBool, AtomicI16, AtomicU32, Ordering};

// TODO: Move somewhere else, maybe to glonax-server
pub struct EcuState {
    /// Frist derivative of the encoder position.
    pub speed: [AtomicI16; 8],
    /// Position of encoder.
    pub position: [AtomicU32; 8],
    motion_lock: AtomicBool,
}

impl EcuState {
    pub fn lock(&self) {
        self.speed[0].store(0, Ordering::Relaxed);
        self.speed[1].store(0, Ordering::Relaxed);
        self.speed[2].store(0, Ordering::Relaxed);
        self.speed[3].store(0, Ordering::Relaxed);
        self.speed[4].store(0, Ordering::Relaxed);
        self.speed[5].store(0, Ordering::Relaxed);
        self.speed[6].store(0, Ordering::Relaxed);
        self.speed[7].store(0, Ordering::Relaxed);

        self.motion_lock.store(true, Ordering::Relaxed);
    }

    #[inline]
    pub fn unlock(&self) {
        self.motion_lock.store(false, Ordering::Relaxed);
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.motion_lock.load(Ordering::Relaxed)
    }
}

impl Default for EcuState {
    fn default() -> Self {
        Self {
            speed: [0; 8].map(|_| AtomicI16::new(0)),
            position: [0; 8].map(|_| AtomicU32::new(0)),
            motion_lock: AtomicBool::new(false),
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
    pub state: RobotState, // TODO: Replace by generic, impl the generic in glonax-server
    /// ECU state.
    pub ecu_state: EcuState, // TODO: Move into robot state
}

impl Default for Operand {
    fn default() -> Self {
        Self {
            status: core::Status::Healthy,
            instance: core::Instance::default(),
            state: RobotState::default(),
        }
    }
}
