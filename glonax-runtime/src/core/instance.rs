use bytes::{BufMut, BytesMut};
use serde_derive::Deserialize;

use super::MachineType;

// TODO: Rename to 'Machine'
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Instance {
    /// Instance unique identifier.
    pub id: uuid::Uuid,
    /// Machine model.
    pub model: String,
    /// Machine machine type.
    pub ty: MachineType,
    /// Machine version.
    pub version: (u8, u8, u8),
}

impl Instance {
    /// Construct new instance.
    pub fn new(
        id: impl ToString,
        model: impl ToString,
        ty: MachineType,
        version: (u8, u8, u8),
    ) -> Self {
        Self {
            id: uuid::Uuid::parse_str(&id.to_string()).unwrap(),
            model: model.to_string(),
            ty,
            version,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(64);

        buf.put(&self.id.as_bytes()[..]);
        buf.put_u8(self.ty as u8);
        buf.put_u8(self.version.0);
        buf.put_u8(self.version.1);
        buf.put_u8(self.version.2);

        let model_bytes = self.model.as_bytes();
        buf.put_u16(model_bytes.len() as u16);
        buf.put(model_bytes);

        buf.to_vec()
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Instance ID: {}, Model: {}, Type: {:?}, Version: {}.{}.{}",
            self.id, self.model, self.ty, self.version.0, self.version.1, self.version.2
        )
    }
}

impl TryFrom<&[u8]> for Instance {
    type Error = ();

    // TODO: Use BytesMut
    fn try_from(buffer: &[u8]) -> std::result::Result<Self, Self::Error> {
        // let mut buf = Bytes::copy_from_slice(value);

        if buffer.len() < 6 {
            log::warn!("Invalid buffer size");
            return Err(());
        }

        let id = uuid::Uuid::from_slice(&buffer[..16]).unwrap();
        let ty = MachineType::try_from(buffer[16]).unwrap();
        let version = (buffer[17], buffer[18], buffer[19]);

        let model_length = u16::from_be_bytes([buffer[20], buffer[21]]) as usize;
        let model = String::from_utf8_lossy(&buffer[22..22 + model_length]).to_string();

        Ok(Self::new(id, model, ty, version))
    }
}

impl TryFrom<Vec<u8>> for Instance {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Instance::try_from(&value[..])
    }
}

impl crate::protocol::Packetize for Instance {
    const MESSAGE_TYPE: u8 = 0x15;

    fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance() {
        let instance = Instance::new(
            "d55bcd75-8d30-49af-ac18-ee7cbce7822f",
            "Test",
            MachineType::Excavator,
            (0, 0, 1),
        );

        let bytes = instance.to_bytes();
        let instance2 = Instance::try_from(&bytes[..]).unwrap();

        assert_eq!(instance, instance2);
    }
}
