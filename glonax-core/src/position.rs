/// 3 axis vector in euler space.
#[derive(Debug)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

/// Free axes of machine rotation.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// Longitudinal axis.
    pub roll: f32,
    /// Transverse axis.
    pub pitch: f32,
    /// Normal axis.
    pub yaw: f32,
}

impl Position {
    pub fn from_raw(x: i32, y: i32, z: i32) -> Position {
        let x = x as f32;
        let y = y as f32;
        let z = z as f32;

        Vector3 { x, y, z }.into()
    }
}

impl From<Vector3<f32>> for Position {
    fn from(value: Vector3<f32>) -> Self {
        Position {
            roll: value.y.atan2(value.z),
            pitch: value.x.atan2(value.z),
            yaw: value.x.atan2(value.y),
        }
    }
}
