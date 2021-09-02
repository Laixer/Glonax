use std::collections::HashMap;

pub struct ActuatorMap(HashMap<u32, u32>);

impl ActuatorMap {
    /// Create new and empty ActuatorMap.
    pub fn new() -> Self {
        ActuatorMap(HashMap::default())
    }

    /// Get the map value or return the input as default.
    pub fn get_or_default(&self, value: u32) -> u32 {
        self.0.get(&value).unwrap_or(&value).clone()
    }

    /// Insert mapping value.
    ///
    /// If the value was already in the map then its updated
    /// and the old value is returned. In all other cases
    /// `None` is returned.
    pub fn insert(&mut self, k: u32, v: u32) -> Option<u32> {
        self.0.insert(k, v)
    }

    /// Flip two actuators.
    ///
    /// After insert the key becomes the value and vice versa.
    /// This is the recommended way to map actuators because it
    /// is a non-reducing operation. All actuators will remain
    /// addressable.
    pub fn insert_bilateral(&mut self, k: u32, v: u32) {
        self.0.insert(k, v);
        self.0.insert(v, k);
    }
}
