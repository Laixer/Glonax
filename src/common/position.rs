use super::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub roll: f32,
    pub pitch: f32,
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
