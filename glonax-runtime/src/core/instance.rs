use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde_derive::Deserialize;

use super::MachineType;

/// Represents an instance of a machine.
///
/// This struct holds information about a machine instance, including its unique identifier,
/// model, type, version, and serial number.
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
    /// Machine serial number.
    serial_number: String,
}

impl Instance {
    /// Construct new instance.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the instance.
    /// * `model` - The model of the instance.
    /// * `ty` - The type of the instance.
    /// * `version` - The version of the instance.
    /// * `serial_number` - The serial number of the instance.
    ///
    /// # Returns
    ///
    /// A new `Instance` object.
    ///
    /// # Examples
    ///
    /// ```
    /// use glonax::core::{Instance, MachineType};
    ///
    /// let instance = Instance::new(
    ///     "2c56e802-fd6b-4401-8f3e-89383f408dec",
    ///     "Model XYZ",
    ///     MachineType::WheelLoader,
    ///     (1, 2, 3),
    ///     "ABC123"
    /// );
    /// ```
    pub fn new(
        id: impl ToString,
        model: impl ToString,
        ty: MachineType,
        version: (u8, u8, u8),
        serial_number: impl ToString,
    ) -> Self {
        Self {
            id: uuid::Uuid::parse_str(&id.to_string()).unwrap(),
            model: model.to_string(),
            ty,
            version,
            serial_number: serial_number.to_string(),
        }
    }

    /// Retrieve the instance unique identifier.
    ///
    /// # Returns
    ///
    /// The unique identifier of the instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use glonax::core::{Instance, MachineType};
    ///
    /// let instance = Instance::new(
    ///    "2c56e802-fd6b-4401-8f3e-89383f408dec",
    ///    "Model XYZ",
    ///    MachineType::WheelLoader,
    ///    (1, 2, 3),
    ///    "ABC123"
    /// );
    ///
    /// assert_eq!(instance.id().to_string(), "2c56e802-fd6b-4401-8f3e-89383f408dec");
    /// ```
    #[inline]
    pub fn id(&self) -> &uuid::Uuid {
        &self.id
    }

    /// Retrieve the instance unique identifier as a short string.
    ///
    /// # Returns
    ///
    /// The unique identifier of the instance as a short string.
    ///
    /// # Examples
    ///
    /// ```
    /// use glonax::core::{Instance, MachineType};
    ///
    /// let instance = Instance::new(
    ///    "2c56e802-fd6b-4401-8f3e-89383f408dec",
    ///    "Model XYZ",
    ///    MachineType::WheelLoader,
    ///    (1, 2, 3),
    ///    "ABC123"
    /// );
    ///
    /// assert_eq!(instance.id_short(), "2c56e802");
    /// ```
    pub fn id_short(&self) -> String {
        self.id.to_string().chars().take(8).collect()
    }

    /// Retrieve the instance model.
    ///
    /// # Returns
    ///
    /// The model of the instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use glonax::core::{Instance, MachineType};
    ///
    /// let instance = Instance::new(
    ///    "2c56e802-fd6b-4401-8f3e-89383f408dec",
    ///    "Model XYZ",
    ///    MachineType::WheelLoader,
    ///    (1, 2, 3),
    ///    "ABC123"
    /// );
    ///
    /// assert_eq!(instance.model(), "Model XYZ");
    #[inline]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Retrieve the instance type.
    ///
    /// # Returns
    ///
    /// The type of the instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use glonax::core::{Instance, MachineType};
    ///
    /// let instance = Instance::new(
    ///    "2c56e802-fd6b-4401-8f3e-89383f408dec",
    ///    "Model XYZ",
    ///    MachineType::WheelLoader,
    ///    (1, 2, 3),
    ///    "ABC123"
    /// );
    ///
    /// assert_eq!(instance.ty(), MachineType::WheelLoader);
    #[inline]
    pub fn ty(&self) -> MachineType {
        self.ty
    }

    /// Retrieve the instance version.
    ///
    /// # Returns
    ///
    /// The version of the instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use glonax::core::{Instance, MachineType};
    ///
    /// let instance = Instance::new(
    ///    "2c56e802-fd6b-4401-8f3e-89383f408dec",
    ///    "Model XYZ",
    ///    MachineType::WheelLoader,
    ///    (1, 2, 3),
    ///    "ABC123"
    /// );
    ///
    /// assert_eq!(instance.version(), (1, 2, 3));
    /// ```
    #[inline]
    pub fn version(&self) -> (u8, u8, u8) {
        self.version
    }

    /// Retrieve the instance version as a string.
    ///
    /// # Returns
    ///
    /// The version of the instance as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use glonax::core::{Instance, MachineType};
    ///
    /// let instance = Instance::new(
    ///    "2c56e802-fd6b-4401-8f3e-89383f408dec",
    ///    "Model XYZ",
    ///    MachineType::WheelLoader,
    ///    (1, 2, 3),
    ///    "ABC123"
    /// );
    ///
    /// assert_eq!(instance.version_string(), "1.2.3");
    #[inline]
    pub fn version_string(&self) -> String {
        format!("{}.{}.{}", self.version.0, self.version.1, self.version.2)
    }

    /// Retrieve the instance serial number.
    ///
    /// # Returns
    ///
    /// The serial number of the instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use glonax::core::{Instance, MachineType};
    ///
    /// let instance = Instance::new(
    ///    "2c56e802-fd6b-4401-8f3e-89383f408dec",
    ///    "Model XYZ",
    ///    MachineType::WheelLoader,
    ///    (1, 2, 3),
    ///    "ABC123"
    /// );
    ///
    /// assert_eq!(instance.serial_number(), "ABC123");
    #[inline]
    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }
}

impl std::fmt::Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Instance ID: {}, Model: {}, Type: {:?}, Version: {}; Serial: {}",
            self.id,
            self.model,
            self.ty,
            self.version_string(),
            self.serial_number
        )
    }
}

impl TryFrom<Vec<u8>> for Instance {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut buf = Bytes::copy_from_slice(&value);

        let id = uuid::Uuid::from_slice(&buf.copy_to_bytes(16)).map_err(|_| ())?;
        let ty = MachineType::try_from(buf.get_u8()).map_err(|_| ())?;
        let version = (buf.get_u8(), buf.get_u8(), buf.get_u8());

        let model_len = buf.get_u16() as usize;
        let model = buf.copy_to_bytes(model_len);

        let serial_len = buf.get_u16() as usize;
        let serial_number = buf.copy_to_bytes(serial_len);

        Ok(Instance {
            id,
            ty,
            version,
            model: String::from_utf8_lossy(&model).to_string(),
            serial_number: String::from_utf8_lossy(&serial_number).to_string(),
        })
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

        let serial_bytes = self.serial_number.as_bytes();
        buf.put_u16(serial_bytes.len() as u16);
        buf.put(serial_bytes);

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
            "T.00001.T.00002",
        );

        let bytes = instance.to_bytes();
        let instance2 = Instance::try_from(bytes).unwrap();

        assert_eq!(instance, instance2);
    }
}
