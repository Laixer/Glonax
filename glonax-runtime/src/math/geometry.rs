use nalgebra::{Rotation3, UnitQuaternion};

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

/// Implements the `EulerAngles` trait for the `UnitQuaternion<f32>` type.
impl EulerAngles for UnitQuaternion<f32> {
    /// Creates a `UnitQuaternion` from a roll angle.
    ///
    /// # Arguments
    ///
    /// * `roll` - The roll angle in radians.
    ///
    /// # Returns
    ///
    /// A `UnitQuaternion` representing the rotation.
    #[inline]
    fn from_roll(roll: f32) -> Self {
        UnitQuaternion::from_euler_angles(roll, 0.0, 0.0)
    }

    /// Creates a `UnitQuaternion` from a pitch angle.
    ///
    /// # Arguments
    ///
    /// * `pitch` - The pitch angle in radians.
    ///
    /// # Returns
    ///
    /// A `UnitQuaternion` representing the rotation.
    #[inline]
    fn from_pitch(pitch: f32) -> Self {
        UnitQuaternion::from_euler_angles(0.0, pitch, 0.0)
    }

    /// Creates a `UnitQuaternion` from a yaw angle.
    ///
    /// # Arguments
    ///
    /// * `yaw` - The yaw angle in radians.
    ///
    /// # Returns
    ///
    /// A `UnitQuaternion` representing the rotation.
    #[inline]
    fn from_yaw(yaw: f32) -> Self {
        UnitQuaternion::from_euler_angles(0.0, 0.0, yaw)
    }
}
