use std::{collections::VecDeque, path::Path};

use glonax::geometry::Target;

pub struct Program(VecDeque<Target>);

impl Program {
    pub fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let str = std::fs::read_to_string(path)?;
        let targets: VecDeque<Target> = serde_json::from_str::<Vec<[f32; 6]>>(&str)?
            .iter()
            .map(|v| v.into())
            .collect();

        Ok(Self(targets))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Target> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn next(&mut self) -> Option<Target> {
        self.0.pop_front()
    }
}

impl FromIterator<Target> for Program {
    fn from_iter<T: IntoIterator<Item = Target>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
