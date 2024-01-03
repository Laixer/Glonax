use std::sync::atomic::{AtomicBool, AtomicI16, AtomicU32, Ordering};

use crate::core::Actuator;

// TODO: This is only used for the simulator, rename
pub struct VirtualHCU {
    /// Frist derivative of the encoder position.
    pub speed: [AtomicI16; 8],
    /// Position of encoder.
    pub position: [AtomicU32; 8],
    /// Motion lock.
    motion_lock: AtomicBool,
}

impl VirtualHCU {
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
    pub fn speed(&self, actuator: &Actuator) -> i16 {
        self.speed[*actuator as usize].load(Ordering::SeqCst)
    }

    #[inline]
    pub fn position(&self, actuator: &Actuator) -> u32 {
        self.position[*actuator as usize].load(Ordering::SeqCst)
    }

    #[inline]
    pub fn set_position(&self, actuator: &Actuator, position: u32) {
        self.position[*actuator as usize].store(position, Ordering::Relaxed);
    }
}

impl Default for VirtualHCU {
    fn default() -> Self {
        Self {
            speed: [0; 8].map(|_| AtomicI16::new(0)),
            position: [0; 8].map(|_| AtomicU32::new(0)),
            motion_lock: AtomicBool::new(false),
        }
    }
}
