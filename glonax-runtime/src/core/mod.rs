pub mod input;
pub mod metric;
pub mod motion;

use std::f32::consts::PI;

/// Level trait.
pub trait Level {
    /// Return the value of self above the lower threshold.
    /// Otherwise return a default value.
    fn ramp(self, lower: Self) -> Self;
}

impl Level for i16 {
    fn ramp(self, lower: Self) -> Self {
        if self < lower && self > -lower {
            0
        } else {
            self
        }
    }
}

pub mod time {
    use std::time::{Duration, SystemTime};

    /// Return the current time as a duration.
    #[inline]
    pub fn now() -> Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
    }
}

/// Convert degree to radian
pub fn deg_to_rad(input: f32) -> f32 {
    input * (PI / 180.0)
}

/// Convert radian to degree
pub fn rad_to_deg(input: f32) -> f32 {
    input * (180.0 / PI)
}

pub trait Identity {
    /// Introduction message.
    ///
    /// Returns a string to introduce the object for the first time and
    /// should only be called once.
    fn intro() -> String;
}
