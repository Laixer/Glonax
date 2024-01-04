use bytes::{BufMut, BytesMut};
use serde_derive::Deserialize;

// TODO: Rename to 'Machine'
// TODO: Change id to uuid
// TODO: Add a machine type
// TODO: Remove name
// TODO: Change model to a integer
// TODO: Include version
#[derive(Clone, Debug, Deserialize, Default)]
pub struct Instance {
    /// Instance unique identifier.
    pub id: String,
    /// Instance model.
    pub model: String,
    /// Instance name.
    pub name: String,
}

impl Instance {
    /// Construct new instance.
    pub fn new(id: impl ToString, model: impl ToString, name: impl ToString) -> Self {
        Self {
            id: id.to_string(),
            model: model.to_string(),
            name: name.to_string(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let id_bytes = self.id.as_bytes();
        let model_bytes = self.model.as_bytes();
        let name_bytes = self.name.as_bytes();

        let mut buf =
            BytesMut::with_capacity(6 + id_bytes.len() + model_bytes.len() + name_bytes.len());

        buf.put_u16(id_bytes.len() as u16);
        buf.put_u16(model_bytes.len() as u16);
        buf.put_u16(name_bytes.len() as u16);
        buf.put(id_bytes);
        buf.put(model_bytes);
        buf.put(name_bytes);

        buf.to_vec()
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Instance ID: {}, Model: {}, Name: {}",
            self.id, self.model, self.name
        )
    }
}

impl TryFrom<&[u8]> for Instance {
    type Error = ();

    fn try_from(buffer: &[u8]) -> std::result::Result<Self, Self::Error> {
        if buffer.len() < 6 {
            log::warn!("Invalid buffer size");
            return Err(());
        }

        let id_length = u16::from_be_bytes([buffer[0], buffer[1]]) as usize;
        let model_length = u16::from_be_bytes([buffer[2], buffer[3]]) as usize;
        let name_length = u16::from_be_bytes([buffer[4], buffer[5]]) as usize;

        let id = String::from_utf8_lossy(&buffer[6..6 + id_length]).to_string();
        let model = String::from_utf8_lossy(&buffer[6 + id_length..6 + id_length + model_length])
            .to_string();
        let name = String::from_utf8_lossy(
            &buffer[6 + id_length + model_length..6 + id_length + model_length + name_length],
        )
        .to_string();

        Ok(Self::new(id, model, name))
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
