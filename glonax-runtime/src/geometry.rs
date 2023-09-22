use nalgebra::{Point3, Rotation3, UnitQuaternion};
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

#[derive(Clone, Copy)]
pub struct Target {
    /// The point in space to move to
    pub point: Point3<f32>,
    /// The orientation to move to
    pub orientation: UnitQuaternion<f32>,
}

impl Target {
    /// Construct a new target
    pub fn new(point: Point3<f32>, orientation: UnitQuaternion<f32>) -> Self {
        Self { point, orientation }
    }

    /// Construct a new target from a point
    pub fn from_point(x: f32, y: f32, z: f32) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::identity(),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({:.2}, {:.2}, {:.2}) [{:.2}rad {:.2}°, {:.2}rad {:.2}°, {:.2}rad {:.2}°]",
            self.point.x,
            self.point.y,
            self.point.z,
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.x * self.orientation.angle()),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.x * self.orientation.angle())
                .to_degrees(),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.y * self.orientation.angle()),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.y * self.orientation.angle())
                .to_degrees(),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.z * self.orientation.angle()),
            self.orientation
                .axis()
                .map_or(0.0, |axis| axis.z * self.orientation.angle())
                .to_degrees(),
        )
    }
}

impl From<(f32, f32, f32)> for Target {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::identity(),
        }
    }
}

impl From<(f32, f32, f32, f32, f32, f32)> for Target {
    fn from((x, y, z, roll, pitch, yaw): (f32, f32, f32, f32, f32, f32)) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::from_euler_angles(roll, pitch, yaw),
        }
    }
}

impl From<[f32; 3]> for Target {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::identity(),
        }
    }
}

impl From<[f32; 6]> for Target {
    fn from([x, y, z, roll, pitch, yaw]: [f32; 6]) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::from_euler_angles(roll, pitch, yaw),
        }
    }
}

impl From<&[f32; 6]> for Target {
    fn from([x, y, z, roll, pitch, yaw]: &[f32; 6]) -> Self {
        Self {
            point: Point3::new(*x, *y, *z),
            orientation: UnitQuaternion::from_euler_angles(*roll, *pitch, *yaw),
        }
    }
}

impl From<Point3<f32>> for Target {
    fn from(point: Point3<f32>) -> Self {
        Self {
            point,
            orientation: UnitQuaternion::identity(),
        }
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
