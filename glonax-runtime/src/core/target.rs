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

impl TryFrom<Vec<u8>> for Target {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        use bytes::Buf;

        let mut buf = bytes::Bytes::copy_from_slice(value.as_slice());

        let point = Point3::new(buf.get_f32(), buf.get_f32(), buf.get_f32());
        let orientation =
            UnitQuaternion::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());

        Ok(Self { point, orientation })
    }
}

impl crate::protocol::Packetize for Target {
    const MESSAGE_TYPE: u8 = 0x44;
    const MESSAGE_SIZE: Option<usize> = Some(std::mem::size_of::<f32>() * 6);

    fn to_bytes(&self) -> Vec<u8> {
        use bytes::BufMut;

        let mut buf = bytes::BytesMut::with_capacity(std::mem::size_of::<f32>() * 6);

        buf.put_f32(self.point.coords[0]);
        buf.put_f32(self.point.coords[1]);
        buf.put_f32(self.point.coords[2]);

        let (roll, pitch, yaw) = self.orientation.euler_angles();
        buf.put_f32(roll);
        buf.put_f32(pitch);
        buf.put_f32(yaw);

        buf.to_vec()
    }
}
