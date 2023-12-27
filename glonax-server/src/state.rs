use std::sync::atomic::{AtomicBool, AtomicI16, AtomicU32, Ordering};

use glonax::{core, runtime::SharedOperandState, RobotState};

pub(crate) type SharedExcavatorState = SharedOperandState<Excavator>;

// TODO: This is only used for the simulator, rename
pub struct EcuState {
    /// Frist derivative of the encoder position.
    pub speed: [AtomicI16; 8],
    /// Position of encoder.
    pub position: [AtomicU32; 8],
    /// Motion lock.
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

    #[inline]
    pub fn speed(&self, actuator: &glonax::core::Actuator) -> i16 {
        self.speed[*actuator as usize].load(Ordering::SeqCst)
    }

    #[inline]
    pub fn position(&self, actuator: &glonax::core::Actuator) -> u32 {
        self.position[*actuator as usize].load(Ordering::SeqCst)
    }

    #[inline]
    pub fn set_position(&self, actuator: &glonax::core::Actuator, position: u32) {
        self.position[*actuator as usize].store(position, Ordering::Relaxed);
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
    // TODO: Move to core state Robot or something
    /// Vehicle management system data.
    pub(crate) vms: core::Host,
    // TODO: Move to core state Robot or something
    /// Global navigation satellite system data.
    pub(crate) gnss: core::Gnss,
    // TODO: Move to core state Robot or something
    /// Engine data.
    pub(crate) engine: core::Engine,
    /// Pose data.
    pub(crate) pose: core::Pose,
    // TODO: Move to core state Robot or something
    /// Electronic control unit data.
    pub(crate) ecu_state: EcuState,
}

impl RobotState for Excavator {
    /// Vehicle management system.
    fn vms_mut(&mut self) -> &mut core::Host {
        &mut self.vms
    }

    /// Global navigation satellite system.
    fn gnss_mut(&mut self) -> &mut core::Gnss {
        &mut self.gnss
    }

    /// Engine management system.
    fn engine_mut(&mut self) -> &mut core::Engine {
        &mut self.engine
    }

    /// Robot pose.
    fn pose_mut(&mut self) -> &mut core::Pose {
        &mut self.pose
    }
}
