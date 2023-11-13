use std::sync::atomic::{AtomicBool, AtomicI16, AtomicU32, Ordering};

use glonax::{core, runtime::SharedOperandState, RobotState};

pub(crate) type SharedExcavatorState = SharedOperandState<Excavator>;

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

#[derive(Default)]
pub struct Excavator {
    /// Vehicle management system data.
    pub(crate) vms: core::Host,
    /// Global navigation satellite system data.
    pub(crate) gnss: core::Gnss,
    /// Engine data.
    pub(crate) engine: core::Engine,
    /// Pose data.
    pub(crate) pose: core::Pose,
    /// Electronic control unit data.
    pub(crate) ecu_state: EcuState,
}

impl Excavator {}

impl RobotState for Excavator {
    fn vms_mut(&mut self) -> &mut core::Host {
        &mut self.vms
    }

    fn gnss_mut(&mut self) -> &mut core::Gnss {
        &mut self.gnss
    }

    fn engine_mut(&mut self) -> &mut core::Engine {
        &mut self.engine
    }

    fn pose_mut(&mut self) -> &mut core::Pose {
        &mut self.pose
    }
}
