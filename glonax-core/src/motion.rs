use std::{ops::Range, u32};

/// Motion instruction.
///
/// Whether or not the instruction has positive effect
/// depends on the motion device itself. The motion device
/// may support more or less functionality to control motion.
///
/// The motion value can communicate the full range of an i16.
/// The signness of the value is often used as a forward/backward
/// motion indicator. However this is left to the motion device.
#[derive(Debug)]
pub enum Motion {
    /// Stop all motion.
    StopAll,
    /// Stop motion on actuators.
    Stop(Vec<u32>),
    /// Change motion on actuators.
    Change(Vec<(u32, i16)>),
}

unsafe impl Sync for Motion {}
unsafe impl Send for Motion {}

pub struct NormalControl {
    /// Actuator.
    pub actuator: u32,
    /// Actuation normal.
    pub value: f32,
    /// Actuation range.
    pub range: Range<i16>,
}

impl NormalControl {
    pub const MAX: f32 = 1.0;
    pub const NIL: f32 = 0.0;
    pub const MIN: f32 = -1.0;

    pub fn new(actuator: u32, value: f32) -> Self {
        Self {
            actuator,
            value,
            ..Default::default()
        }
    }

    /// Convert normal to effective range.
    ///
    /// If the unbound range is outside the absolute
    /// range it is rounded to the range upperound.
    ///
    /// The `DEAD_VALUE` constitudes a measurement error.
    /// Any value below this constant is interpreted as 0.
    pub fn to_motion(&self) -> Motion {
        const DEAD_VALUE: f32 = 0.02;

        if self.value.abs() < DEAD_VALUE {
            Motion::Stop(vec![self.actuator])
        } else {
            let unbound_range = (self.value * (self.range.end - self.range.start) as f32) as i16;
            let value = if self.value.is_sign_positive() {
                self.range.end.min(unbound_range + self.range.start)
            } else {
                // FUTURE: use min(..)
                let value = unbound_range - self.range.start;
                if value < -self.range.end {
                    -self.range.end
                } else {
                    value
                }
            };

            Motion::Change(vec![(self.actuator, value)])
        }
    }
}

impl From<NormalControl> for Motion {
    fn from(value: NormalControl) -> Self {
        value.to_motion()
    }
}

impl Default for NormalControl {
    fn default() -> Self {
        NormalControl {
            actuator: 0,
            value: NormalControl::NIL,
            range: 150..256,
        }
    }
}