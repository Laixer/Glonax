use nalgebra::{Rotation3, UnitVector3, Vector};

struct EncoderAdapter {
    /// Encoder factor.
    factor: f32,
    /// Encoder offset.
    offset: f32,
    /// Invert encoder.
    invert: bool,
    /// Encoder axis.
    axis: UnitVector3<f32>,
}

impl EncoderAdapter {
    fn as_radians(&self, position: u32) -> f32 {
        let position = (position as f32 / self.factor) - self.offset;
        if self.invert {
            position * -1.0
        } else {
            position
        }
    }

    #[inline]
    fn as_rotation(&self, position: u32) -> Rotation3<f32> {
        Rotation3::from_axis_angle(&self.axis, self.as_radians(position))
    }
}

pub struct Pose {
    /// Frame rotator.
    frame_rotator: Rotation3<f32>,
    /// Boom rotator.
    boom_rotator: Rotation3<f32>,
    /// Arm rotator.
    arm_rotator: Rotation3<f32>,
    /// Attachment rotator.
    attachment_rotator: Rotation3<f32>,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            frame_rotator: Rotation3::identity(),
            boom_rotator: Rotation3::identity(),
            arm_rotator: Rotation3::identity(),
            attachment_rotator: Rotation3::identity(),
        }
    }
}

impl Pose {
    pub fn set_node_position(&mut self, node: u8, position: u32) {
        match node {
            0x6A => {
                let frame_encoder = EncoderAdapter {
                    factor: 1000.0,
                    offset: 0.0,
                    invert: true,
                    axis: Vector::z_axis(),
                };

                self.frame_rotator = frame_encoder.as_rotation(position);
            }
            0x6B => {
                let offset = 60_f32.to_radians();
                let position = position as f32 / 1000.0;
                let position = (position - offset) * -1.0;
                self.boom_rotator = Rotation3::from_euler_angles(0.0, position, 0.0);
            }
            0x6C => {
                let arm_encoder = EncoderAdapter {
                    factor: 1000.0,
                    offset: 0.0,
                    invert: true,
                    axis: Vector::y_axis(),
                };

                self.arm_rotator = arm_encoder.as_rotation(position);
            }
            0x6D => {
                let attachment_encoder = EncoderAdapter {
                    factor: 1000.0,
                    offset: 0.0,
                    invert: true,
                    axis: Vector::y_axis(),
                };

                self.attachment_rotator = attachment_encoder.as_rotation(position);
            }
            _ => {}
        }
    }

    // TODO: Copy into bytes directly
    pub fn to_bytes(&self) -> Vec<u8> {
        use bytes::BufMut;

        let mut buf = bytes::BytesMut::with_capacity(32);

        let (roll, pitch, yaw) = self.frame_rotator.euler_angles();
        buf.put_f32(roll);
        buf.put_f32(pitch);
        buf.put_f32(yaw);

        let (roll, pitch, yaw) = self.boom_rotator.euler_angles();
        buf.put_f32(roll);
        buf.put_f32(pitch);
        buf.put_f32(yaw);

        let (roll, pitch, yaw) = self.arm_rotator.euler_angles();
        buf.put_f32(roll);
        buf.put_f32(pitch);
        buf.put_f32(yaw);

        let (roll, pitch, yaw) = self.attachment_rotator.euler_angles();
        buf.put_f32(roll);
        buf.put_f32(pitch);
        buf.put_f32(yaw);

        buf.to_vec()
    }
}

impl TryFrom<&[u8]> for Pose {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use bytes::Buf;

        let mut buf = bytes::Bytes::copy_from_slice(value);

        let frame_rotator =
            Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());
        let boom_rotator =
            Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());
        let arm_rotator = Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());
        let attachment_rotator =
            Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());

        Ok(Self {
            frame_rotator,
            boom_rotator,
            arm_rotator,
            attachment_rotator,
        })
    }
}

impl std::fmt::Display for Pose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        let (_roll, _pitch, yaw) = self.frame_rotator.euler_angles();
        s.push_str(&format!("Frame: {:.2}rad {:.2}°; ", yaw, yaw.to_degrees(),));
        let (_roll, pitch, _yaw) = self.boom_rotator.euler_angles();
        s.push_str(&format!(
            "Boom: {:.2}rad {:.2}°; ",
            pitch,
            pitch.to_degrees()
        ));
        let (_roll, pitch, _yaw) = self.arm_rotator.euler_angles();
        s.push_str(&format!(
            "Arm: {:.2}rad {:.2}°; ",
            pitch,
            pitch.to_degrees()
        ));
        let (_roll, pitch, _yaw) = self.attachment_rotator.euler_angles();
        s.push_str(&format!(
            "Attachment: {:.2}rad {:.2}°",
            pitch,
            pitch.to_degrees()
        ));

        write!(f, "{}", s)
    }
}

impl TryFrom<Vec<u8>> for Pose {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Pose::try_from(&value[..])
    }
}

impl crate::transport::Packetize for Pose {
    const MESSAGE: crate::transport::frame::FrameMessage =
        crate::transport::frame::FrameMessage::Pose;
    const MESSAGE_SIZE: Option<usize> = Some(std::mem::size_of::<f32>() * 12);

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}
