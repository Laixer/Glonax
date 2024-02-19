use bytes::{BufMut, BytesMut};
use serde_derive::Deserialize;

use super::MachineType;

// TODO: Rename to 'Machine'
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Instance {
    /// Instance unique identifier.
    id: uuid::Uuid,
    /// Machine model.
    model: String,
    /// Machine machine type.
    ty: MachineType,
    /// Machine version.
    version: (u8, u8, u8),
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

    /// Retrieve the instance unique identifier.
    #[inline]
    pub fn id(&self) -> &uuid::Uuid {
        &self.id
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

impl TryFrom<Vec<u8>> for Instance {
    type Error = ();

    // TODO: Use BytesMut
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // let mut buf = Bytes::copy_from_slice(value);

        if value.len() < 6 {
            log::warn!("Invalid buffer size");
            return Err(());
        }

        let id = uuid::Uuid::from_slice(&value[..16]).unwrap();
        let ty = MachineType::try_from(value[16]).unwrap();
        let version = (value[17], value[18], value[19]);

        let model_length = u16::from_be_bytes([value[20], value[21]]) as usize;
        let model = String::from_utf8_lossy(&value[22..22 + model_length]).to_string();

        Ok(Self::new(id, model, ty, version))
    }
}

impl crate::protocol::Packetize for Instance {
    const MESSAGE_TYPE: u8 = 0x15;

    fn to_bytes(&self) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use crate::protocol::Packetize;

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
        let instance2 = Instance::try_from(bytes).unwrap();

        assert_eq!(instance, instance2);
    }
}
