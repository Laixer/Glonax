pub struct Pose {
    /// Frame rotator.
    pub frame_rotator: nalgebra::Rotation3<f32>,
    /// Boom rotator.
    pub boom_rotator: nalgebra::Rotation3<f32>,
    /// Arm rotator.
    pub arm_rotator: nalgebra::Rotation3<f32>,
    /// Attachment rotator.
    pub attachment_rotator: nalgebra::Rotation3<f32>,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            frame_rotator: nalgebra::Rotation3::identity(),
            boom_rotator: nalgebra::Rotation3::identity(),
            arm_rotator: nalgebra::Rotation3::identity(),
            attachment_rotator: nalgebra::Rotation3::identity(),
        }
    }
}

impl Pose {
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
            nalgebra::Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());
        let boom_rotator =
            nalgebra::Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());
        let arm_rotator =
            nalgebra::Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());
        let attachment_rotator =
            nalgebra::Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32());

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

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}
