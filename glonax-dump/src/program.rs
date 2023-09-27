use std::{collections::VecDeque, path::Path};

use glonax::geometry::Target;

pub type Program = VecDeque<Target>;

pub fn from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Program> {
    let str = std::fs::read_to_string(path)?;

    Ok(serde_json::from_str::<Vec<[f32; 6]>>(&str)?
        .iter()
        .map(|v| v.into())
        .collect())
}
