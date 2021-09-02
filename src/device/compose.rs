use std::{
    collections::{
        hash_map::{Iter, IterMut},
        HashMap,
    },
    ops::{Index, IndexMut},
};

use crate::common::index::{TryIndex, TryIndexMut};

use super::{Device, MetricDevice}; //, MotionDevice};

// const ACTUATORS_PER_CONTROLLER: u32 = 3;

/// Compose a virtual device.
///
/// The virtual composer encapsulates a map of devices. It then
/// presents a single facade to the caller.
pub struct Composer<D> {
    /// Internal list of `D` devices.
    list: HashMap<u32, D>,
    /// Next index key.
    idx: u32,
}

impl<D> Composer<D> {
    /// Create a new empty `ComposeDevice<D>`.
    #[inline]
    pub fn new() -> Self {
        Self::with_index(0)
    }

    /// Create a new empty `ComposeDevice<D>` with a preset index.
    pub fn with_index(idx: u32) -> Self {
        Self {
            list: HashMap::new(),
            idx,
        }
    }

    /// Returns the number of devices owned by the composer.
    #[inline]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Returns `true` if the composer contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Register device with composer.
    ///
    /// The devices will be assigned an index according
    /// to the order of insertion. The assigned index is
    /// returned.
    pub fn insert(&mut self, device: D) -> u32 {
        let key = self.idx;
        self.list.insert(key, device);
        self.idx += 1;
        key
    }

    /// Returns a reference to the value corresponding to the index.
    #[inline]
    pub fn get(&self, idx: u32) -> Option<&D> {
        self.list.get(&idx)
    }

    /// Returns a mutable reference to the value corresponding to the index.
    #[inline]
    pub fn get_mut(&mut self, idx: u32) -> Option<&mut D> {
        self.list.get_mut(&idx)
    }

    // /// An iterator visiting all index-device pairs in order.
    pub fn iter(&self) -> Iter<u32, D> {
        self.list.iter()
    }

    /// An iterator visiting all index-device pairs in order.
    pub fn iter_mut(&mut self) -> IterMut<u32, D> {
        self.list.iter_mut()
    }
}

impl<D> Index<u32> for Composer<D> {
    type Output = D;

    fn index(&self, index: u32) -> &Self::Output {
        self.get(index).expect("No device found for index")
    }
}

impl<D> IndexMut<u32> for Composer<D> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.get_mut(index).expect("No device found for index")
    }
}

impl<D> TryIndex<u32> for Composer<D> {
    type Output = D;

    fn try_index(&self, index: u32) -> Option<&Self::Output> {
        self.get(index)
    }
}

impl<D> TryIndexMut<u32> for Composer<D> {
    fn try_index_mut(&mut self, index: u32) -> Option<&mut Self::Output> {
        self.get_mut(index)
    }
}

impl<D> Default for Composer<D> {
    /// Creates a new `Composer<D>` using [`new`].
    ///
    /// [`new`]: Composer::new
    fn default() -> Self {
        Self::new()
    }
}

impl<D> Device for Composer<D> {
    fn name(&self) -> String {
        "compose device".to_owned()
    }

    // TODO: Impl probe.
}

// TODO: Maybe remove ?
// impl<D: MotionDevice> MotionDevice for Composer<D> {
//     fn actuate(&mut self, actuator: u32, value: i16) {
//         match self.list.get_mut(&(actuator / ACTUATORS_PER_CONTROLLER)) {
//             Some(device) => device.actuate(actuator % ACTUATORS_PER_CONTROLLER, value),
//             None => warn!("Requested actuator not found"), // TODO: return Err(..)
//         }
//     }

//     fn halt(&mut self) {
//         for device in self.list.values_mut() {
//             device.halt()
//         }
//     }
// }

impl<D: MetricDevice> MetricDevice for Composer<D> {
    // TODO: Filter outliers, return some aggregate.
    fn next(&mut self) -> Option<super::MetricValue> {
        for device in self.list.values_mut() {
            let value = device.next();
            if value.is_some() {
                return value;
            }
        }
        None
    }
}

impl<'a, D> IntoIterator for &'a Composer<D> {
    type Item = (&'a u32, &'a D);

    type IntoIter = Iter<'a, u32, D>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D> IntoIterator for &'a mut Composer<D> {
    type Item = (&'a u32, &'a mut D);

    type IntoIter = IterMut<'a, u32, D>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
