pub use self::engine::Engine;
pub use self::gnss::Gnss;
pub use self::host::Host;
pub use self::instance::Instance;
pub use self::motion::Actuator; // TODO: maybe access via motion::Actuator
pub use self::motion::Motion;
pub use self::pose::Pose;
pub use self::status::Status;

mod engine;
mod gnss;
mod host;
mod instance;
mod motion;
mod pose;
mod status;

pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub type Rotator = Vector3;

impl Rotator {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn identity() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn from_roll(roll: f32) -> Self {
        Self {
            x: roll,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn from_pitch(pitch: f32) -> Self {
        Self {
            x: 0.0,
            y: pitch,
            z: 0.0,
        }
    }

    pub fn from_yaw(yaw: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: yaw,
        }
    }

    pub fn new_x(x: f32) -> Self {
        Self { x, y: 0.0, z: 0.0 }
    }

    pub fn new_y(y: f32) -> Self {
        Self { x: 0.0, y, z: 0.0 }
    }

    pub fn new_z(z: f32) -> Self {
        Self { x: 0.0, y: 0.0, z }
    }
}
