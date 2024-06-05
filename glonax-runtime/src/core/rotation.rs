use nalgebra::Rotation3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RotationReference {
    Absolute,
    Relative,
}

impl TryFrom<u8> for RotationReference {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Absolute),
            1 => Ok(Self::Relative),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rotator {
    pub rotator: Rotation3<f32>,
    pub reference: RotationReference,
}

impl Default for Rotator {
    fn default() -> Self {
        Self {
            rotator: Rotation3::identity(),
            reference: RotationReference::Relative,
        }
    }
}

impl Rotator {
    /// Construct a new target
    pub fn new(rotator: Rotation3<f32>, reference: RotationReference) -> Self {
        Self { rotator, reference }
    }

    /// Construct a new target with an absolute reference
    pub fn absolute(rotator: Rotation3<f32>) -> Self {
        Self {
            rotator,
            reference: RotationReference::Absolute,
        }
    }

    /// Construct a new target with a relative reference
    pub fn relative(rotator: Rotation3<f32>) -> Self {
        Self {
            rotator,
            reference: RotationReference::Relative,
        }
    }
}

impl std::fmt::Display for Rotator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.rotator, self.reference)
    }
}

impl TryFrom<Vec<u8>> for Rotator {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        use bytes::Buf;

        let mut buf = bytes::Bytes::copy_from_slice(value.as_slice());

        Ok(Self {
            rotator: Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32()),
            reference: RotationReference::try_from(buf.get_u8()).unwrap(),
        })
    }
}

impl crate::protocol::Packetize for Rotator {
    const MESSAGE_TYPE: u8 = 0x44;
    const MESSAGE_SIZE: Option<usize> = Some((std::mem::size_of::<f32>() * 6) + 1);

    fn to_bytes(&self) -> Vec<u8> {
        use bytes::BufMut;

        let mut buf = bytes::BytesMut::with_capacity((std::mem::size_of::<f32>() * 6) + 1);

        let (roll, pitch, yaw) = self.rotator.euler_angles();
        buf.put_f32(roll);
        buf.put_f32(pitch);
        buf.put_f32(yaw);

        buf.put_u8(self.reference as u8);

        buf.to_vec()
    }
}
