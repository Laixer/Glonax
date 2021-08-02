use std::{ops::Range, u32};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Actuator {
    Boom = 0,
    Arm = 1,
    Bucket = 2,
    Slew = 3,
    LimpLeft = 4,
    LimpRight = 5,
}

impl From<Actuator> for u32 {
    fn from(actuator: Actuator) -> Self {
        actuator as u32
    }
}

pub trait ToMotionControl {
    /// Returns `MotionControl` from the underlaying type.
    fn to_motion_control(&self) -> MotionControl;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionControl {
    /// Actuator index.
    pub actuator: u32,
    /// Actuation value.
    pub value: i16,
}

unsafe impl Sync for MotionControl {}
unsafe impl Send for MotionControl {}

impl MotionControl {
    // TODO: decouple `Actuator` from `MotionControl`.
    pub fn new(actuator: Actuator, value: i16) -> Self {
        MotionControl {
            actuator: actuator.into(),
            value: if value.abs() < 50 { 0 } else { value },
        }
    }
}

impl ToMotionControl for MotionControl {
    /// Returns `MotionControl` from self.
    ///
    /// This is essentially a noop.
    fn to_motion_control(&self) -> MotionControl {
        *self
    }
}

pub struct AbsControl {
    pub actuator: Actuator,
    pub value: i16,
}

impl AbsControl {
    pub const MAX: i16 = 255;
    pub const NIL: i16 = 0;
    pub const MIN: i16 = -255;
}

impl ToMotionControl for AbsControl {
    fn to_motion_control(&self) -> MotionControl {
        MotionControl::new(self.actuator, self.value)
    }
}

pub struct NormalControl {
    /// Actuator.
    pub actuator: Actuator,
    /// Actuation normal.
    pub value: f32,
    /// Actuation range.
    pub range: Range<i16>,
}

impl NormalControl {
    pub const MAX: f32 = 1.0;
    pub const NIL: f32 = 0.0;
    pub const MIN: f32 = -1.0;
}

impl Default for NormalControl {
    fn default() -> Self {
        NormalControl {
            actuator: Actuator::Arm,
            value: NormalControl::NIL,
            range: 150..256,
        }
    }
}

impl ToMotionControl for NormalControl {
    /// Convert normal to effective range.
    ///
    /// If the unbound range is outside the absolute
    /// range it is rounded to the range upperound.
    ///
    /// The `DEAD_VALUE` constitudes a measurement error.
    /// Any value below this constant is interpreted as 0.
    fn to_motion_control(&self) -> MotionControl {
        const DEAD_VALUE: f32 = 0.02;
        let value = if self.value.abs() < DEAD_VALUE {
            0
        } else {
            let unbound_range = (self.value * (self.range.end - self.range.start) as f32) as i16;
            if self.value.is_sign_positive() {
                self.range.end.min(unbound_range + self.range.start)
            } else {
                // FUTURE: use min(..)
                let value = unbound_range - self.range.start;
                if value < -self.range.end {
                    -self.range.end
                } else {
                    value
                }
            }
        };

        MotionControl::new(self.actuator, value)
    }
}
