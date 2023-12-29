use nalgebra::{Point3, UnitQuaternion};

#[derive(Clone, Copy)]
pub enum Constraint {
    /// Unconstrained motion order.
    Unconstrained = 0,
    /// Motion control of the attachment is delayed until all other controllers have completed
    DelayAttachment = 1,
    /// Motion control will ignore the attachment.
    StationaryAttachment = 2,
    /// Linear priority.
    LinearPriority = 20,
    /// Lateral priority.
    LateralPriority = 21,
    /// Vertical priority.
    VerticalPriority = 22,
}

#[derive(Clone, Copy)]
pub struct Target {
    /// The point in space.
    pub point: Point3<f32>,
    /// The orientation of the target.
    pub orientation: UnitQuaternion<f32>,
    /// The motion constraint.
    pub constraint: Constraint,
}

impl Target {
    /// Construct a new target
    pub fn new(
        point: Point3<f32>,
        orientation: UnitQuaternion<f32>,
        constraint: Constraint,
    ) -> Self {
        Self {
            point,
            orientation,
            constraint,
        }
    }

    /// Construct a new target from a point
    pub fn from_point(x: f32, y: f32, z: f32) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::identity(),
            constraint: Constraint::Unconstrained,
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
            constraint: Constraint::Unconstrained,
        }
    }
}

impl From<(f32, f32, f32, f32, f32, f32)> for Target {
    fn from((x, y, z, roll, pitch, yaw): (f32, f32, f32, f32, f32, f32)) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::from_euler_angles(roll, pitch, yaw),
            constraint: Constraint::Unconstrained,
        }
    }
}

impl From<[f32; 3]> for Target {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::identity(),
            constraint: Constraint::Unconstrained,
        }
    }
}

impl From<[f32; 6]> for Target {
    fn from([x, y, z, roll, pitch, yaw]: [f32; 6]) -> Self {
        Self {
            point: Point3::new(x, y, z),
            orientation: UnitQuaternion::from_euler_angles(roll, pitch, yaw),
            constraint: Constraint::Unconstrained,
        }
    }
}

impl From<&[f32; 6]> for Target {
    fn from([x, y, z, roll, pitch, yaw]: &[f32; 6]) -> Self {
        Self {
            point: Point3::new(*x, *y, *z),
            orientation: UnitQuaternion::from_euler_angles(*roll, *pitch, *yaw),
            constraint: Constraint::Unconstrained,
        }
    }
}

impl From<Point3<f32>> for Target {
    fn from(point: Point3<f32>) -> Self {
        Self {
            point,
            orientation: UnitQuaternion::identity(),
            constraint: Constraint::Unconstrained,
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

        Ok(Self {
            point,
            orientation,
            constraint: match buf.get_u8() {
                0 => Constraint::Unconstrained,
                1 => Constraint::DelayAttachment,
                2 => Constraint::StationaryAttachment,
                20 => Constraint::LinearPriority,
                21 => Constraint::LateralPriority,
                22 => Constraint::VerticalPriority,
                _ => return Err(()),
            },
        })
    }
}

impl crate::protocol::Packetize for Target {
    const MESSAGE_TYPE: u8 = 0x44;
    const MESSAGE_SIZE: Option<usize> = Some((std::mem::size_of::<f32>() * 6) + 1);

    fn to_bytes(&self) -> Vec<u8> {
        use bytes::BufMut;

        let mut buf = bytes::BytesMut::with_capacity((std::mem::size_of::<f32>() * 6) + 1);

        buf.put_f32(self.point.coords[0]);
        buf.put_f32(self.point.coords[1]);
        buf.put_f32(self.point.coords[2]);

        let (roll, pitch, yaw) = self.orientation.euler_angles();
        buf.put_f32(roll);
        buf.put_f32(pitch);
        buf.put_f32(yaw);

        buf.put_u8(self.constraint as u8);

        buf.to_vec()
    }
}
