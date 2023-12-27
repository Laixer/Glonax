use nalgebra::{Rotation3, UnitQuaternion};
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

pub trait EulerAngles {
    /// Create a rotation matrix from a roll angle.
    fn from_roll(roll: f32) -> Self;
    /// Create a rotation matrix from a pitch angle.
    fn from_pitch(pitch: f32) -> Self;
    /// Create a rotation matrix from a yaw angle.
    fn from_yaw(pitch: f32) -> Self;
}

impl EulerAngles for Rotation3<f32> {
    fn from_roll(roll: f32) -> Self {
        Rotation3::from_euler_angles(roll, 0.0, 0.0)
    }

    fn from_pitch(pitch: f32) -> Self {
        Rotation3::from_euler_angles(0.0, pitch, 0.0)
    }

    fn from_yaw(yaw: f32) -> Self {
        Rotation3::from_euler_angles(0.0, 0.0, yaw)
    }
}

impl EulerAngles for UnitQuaternion<f32> {
    fn from_roll(roll: f32) -> Self {
        UnitQuaternion::from_euler_angles(roll, 0.0, 0.0)
    }

    fn from_pitch(pitch: f32) -> Self {
        UnitQuaternion::from_euler_angles(0.0, pitch, 0.0)
    }

    fn from_yaw(yaw: f32) -> Self {
        UnitQuaternion::from_euler_angles(0.0, 0.0, yaw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortest_rotation() {
        assert!(shortest_rotation(45.0_f32.to_radians()) < 46.0_f32.to_radians());
        assert!(shortest_rotation(179.0_f32.to_radians()) < 180.0_f32.to_radians());

        // TODO: More tests
    }
}
