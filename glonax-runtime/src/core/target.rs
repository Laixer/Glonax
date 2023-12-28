use nalgebra::{Point3, UnitQuaternion};

#[derive(Clone, Copy)]
pub struct Target {
    /// The point in space.
    pub point: Point3<f32>,
    /// The orientation of the target.
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
