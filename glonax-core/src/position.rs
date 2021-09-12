use std::f32::consts;

/// 3 axis vector in euler space.
#[derive(Debug)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

/// Free axes of machine rotation.
#[derive(Clone, Copy)]
pub struct Position {
    /// Longitudinal axis.
    pub roll: f32,
    /// Transverse axis.
    pub pitch: f32,
    /// Normal axis.
    pub yaw: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            roll: y.atan2(z),
            pitch: x.atan2(z),
            yaw: x.atan2(y),
        }
    }

    /// Get the roll in degrees.
    pub fn roll_degree(&self) -> f32 {
        (180.0 / consts::PI) * self.roll
    }

    /// Get the pitch in degrees.
    pub fn pitch_degree(&self) -> f32 {
        (180.0 / consts::PI) * self.pitch
    }

    /// Get the yaw in degrees.
    pub fn yaw_degree(&self) -> f32 {
        (180.0 / consts::PI) * self.yaw
    }
}

impl From<&Vector3<i16>> for Position {
    fn from(value: &Vector3<i16>) -> Self {
        Self::new(value.x as f32, value.y as f32, value.z as f32)
    }
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Position: Roll: {:+.3}rad ({:+.1}°) Pitch: {:+.3}rad ({:+.1}°) Yaw: {:+.3}rad ({:+.1}°)",
            self.roll,
            self.roll_degree(),
            self.pitch,
            self.pitch_degree(),
            self.yaw,
            self.yaw_degree()
        )
    }
}
