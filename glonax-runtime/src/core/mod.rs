pub use self::instance::Instance;
pub use self::signal::Metric;
pub use self::signal::Signal;

pub use self::motion::Actuator; // TODO: maybe access via motion::Actuator
pub use self::motion::Motion;

mod instance;
mod motion;
mod signal;

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
    ///
    /// This duration is the time since the UNIX epoch in cooridnated
    /// universal time (UTC).
    #[inline]
    pub fn now() -> Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
    }
}

pub mod geometry {
    use std::f32::consts::PI;

    /// Calculate the shortest rotation between two points on a circle
    pub fn shortest_rotation(distance: f32) -> f32 {
        let dist_normal = (distance + (2.0 * PI)) % (2.0 * PI);

        if dist_normal > PI {
            dist_normal - (2.0 * PI)
        } else {
            dist_normal
        }
    }

    /// Calculate the angle of a triangle using the law of cosines
    pub fn law_of_cosines(a: f32, b: f32, c: f32) -> f32 {
        let a2 = a.powi(2);
        let b2 = b.powi(2);
        let c2 = c.powi(2);

        let numerator = a2 + b2 - c2;
        let denominator = 2.0 * a * b;

        (numerator / denominator).acos()
    }

    /// Convert degrees to radians
    #[inline]
    pub fn deg_to_rad<T: std::ops::Mul<f32, Output = T>>(input: T) -> T {
        input * (std::f32::consts::PI / 180.0)
    }

    /// Convert radians to degrees
    #[inline]
    pub fn rad_to_deg<T: std::ops::Mul<f32, Output = T>>(input: T) -> T {
        input * (180.0 / std::f32::consts::PI)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_shortest_rotation() {
            assert!(shortest_rotation(deg_to_rad(45.0)) < deg_to_rad(46.0));
            assert!(shortest_rotation(deg_to_rad(179.0)) < deg_to_rad(180.0));

            // TODO: More tests
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp() {
        assert_eq!(120_i16.ramp(3072), 0);
        assert_eq!(20_000_i16.ramp(3072), 20_000);
        assert_eq!(-10_i16.ramp(3072), 0);
        assert_eq!(-5960_i16.ramp(3072), -5960);
    }
}
