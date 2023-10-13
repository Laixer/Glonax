use crate::core::Rotator;

pub struct Pose {
    /// Frame rotator.
    pub frame_rotator: Rotator,
    /// Boom rotator.
    pub boom_rotator: Rotator,
    /// Arm rotator.
    pub arm_rotator: Rotator,
    /// Attachment rotator.
    pub attachment_rotator: Rotator,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            frame_rotator: Rotator::identity(),
            boom_rotator: Rotator::identity(),
            arm_rotator: Rotator::identity(),
            attachment_rotator: Rotator::identity(),
        }
    }
}

impl Pose {
    // TODO: Copy into bytes directly
    pub fn to_bytes(&self) -> Vec<u8> {
        use bytes::BufMut;

        let mut buf = bytes::BytesMut::with_capacity(32);

        buf.put_f32(self.frame_rotator.x);
        buf.put_f32(self.frame_rotator.y);
        buf.put_f32(self.frame_rotator.z);

        buf.put_f32(self.boom_rotator.x);
        buf.put_f32(self.boom_rotator.y);
        buf.put_f32(self.boom_rotator.z);

        buf.put_f32(self.arm_rotator.x);
        buf.put_f32(self.arm_rotator.y);
        buf.put_f32(self.arm_rotator.z);

        buf.put_f32(self.attachment_rotator.x);
        buf.put_f32(self.attachment_rotator.y);
        buf.put_f32(self.attachment_rotator.z);

        buf.to_vec()
    }
}

impl TryFrom<&[u8]> for Pose {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use bytes::Buf;

        let mut buf = bytes::Bytes::copy_from_slice(value);

        let frame_rotator = Rotator {
            x: buf.get_f32(),
            y: buf.get_f32(),
            z: buf.get_f32(),
        };

        let boom_rotator = Rotator {
            x: buf.get_f32(),
            y: buf.get_f32(),
            z: buf.get_f32(),
        };
        let arm_rotator = Rotator {
            x: buf.get_f32(),
            y: buf.get_f32(),
            z: buf.get_f32(),
        };
        let attachment_rotator = Rotator {
            x: buf.get_f32(),
            y: buf.get_f32(),
            z: buf.get_f32(),
        };

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

        s.push_str(&format!(
            "Frame: {:.2}rad {:.2}°; ",
            self.frame_rotator.z,
            self.frame_rotator.z.to_degrees(),
        ));
        s.push_str(&format!(
            "Boom: {:.2}rad {:.2}°; ",
            self.boom_rotator.y,
            self.boom_rotator.y.to_degrees(),
        ));
        s.push_str(&format!(
            "Arm: {:.2}rad {:.2}°; ",
            self.arm_rotator.y,
            self.arm_rotator.y.to_degrees(),
        ));
        s.push_str(&format!(
            "Attachment: {:.2}rad {:.2}°",
            self.attachment_rotator.y,
            self.attachment_rotator.y.to_degrees(),
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

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}
