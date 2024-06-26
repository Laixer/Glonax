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

/// Represents a rotator that stores rotation information.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rotator {
    /// The source of the rotation.
    pub source: u8,
    /// The actual rotation.
    pub rotator: Rotation3<f32>,
    /// The reference for the rotation.
    pub reference: RotationReference,
}

impl Rotator {
    /// Construct a new target with an absolute reference
    pub fn absolute(source: u8, rotator: Rotation3<f32>) -> Self {
        Self {
            source,
            rotator,
            reference: RotationReference::Absolute,
        }
    }

    /// Construct a new target with a relative reference
    pub fn relative(source: u8, rotator: Rotation3<f32>) -> Self {
        Self {
            source,
            rotator,
            reference: RotationReference::Relative,
        }
    }
}

impl std::fmt::Display for Rotator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{:X} {:?} Roll={:.2} Pitch={:.2} Yaw={:.2}",
            self.source,
            self.reference,
            self.rotator.euler_angles().0.to_degrees(),
            self.rotator.euler_angles().1.to_degrees(),
            self.rotator.euler_angles().2.to_degrees()
        )
    }
}

impl TryFrom<Vec<u8>> for Rotator {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        use bytes::Buf;

        let mut buf = bytes::Bytes::copy_from_slice(value.as_slice());

        Ok(Self {
            source: buf.get_u8(),
            rotator: Rotation3::from_euler_angles(buf.get_f32(), buf.get_f32(), buf.get_f32()),
            reference: RotationReference::try_from(buf.get_u8()).unwrap(),
        })
    }
}

impl crate::protocol::Packetize for Rotator {
    const MESSAGE_TYPE: u8 = 0x46;
    const MESSAGE_SIZE: Option<usize> = Some((std::mem::size_of::<f32>() * 3) + 1 + 1);

    fn to_bytes(&self) -> Vec<u8> {
        use bytes::BufMut;

        let mut buf = bytes::BytesMut::with_capacity((std::mem::size_of::<f32>() * 3) + 1 + 1);

        buf.put_u8(self.source);

        let (roll, pitch, yaw) = self.rotator.euler_angles();
        buf.put_f32(roll);
        buf.put_f32(pitch);
        buf.put_f32(yaw);

        buf.put_u8(self.reference as u8);

        buf.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Packetize;

    #[test]
    fn test_rotator() {
        let rotator = Rotator {
            source: 0x01,
            rotator: Rotation3::from_euler_angles(0.1, 0.2, 0.3),
            reference: RotationReference::Relative,
        };

        let bytes = rotator.to_bytes();

        assert_eq!(bytes.len(), 14);
        assert_eq!(bytes[13], 0x01);

        let rotator = Rotator::try_from(bytes).unwrap();

        assert_eq!(rotator.source, 0x01);
        assert!((rotator.rotator.euler_angles().0 - 0.1).abs() < f32::EPSILON);
        assert!((rotator.rotator.euler_angles().1 - 0.2).abs() < f32::EPSILON);
        assert!((rotator.rotator.euler_angles().2 - 0.3).abs() < f32::EPSILON);
        assert_eq!(rotator.reference, RotationReference::Relative);
    }
}
