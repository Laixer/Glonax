pub use self::control::Control;
pub use self::engine::{Engine, EngineState};
pub use self::gnss::{Gnss, GnssStatus};
pub use self::host::{Host, HostStatus};
pub use self::instance::Instance;
pub use self::motion::Actuator;
pub use self::motion::Motion;
pub use self::status::Status;
pub use self::target::Target;

mod control;
mod engine;
mod gnss;
mod host;
mod instance;
mod motion;
mod status;
mod target;

// TODO: Add object for encoder
#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    /// Control.
    Control(Control),
    /// Engine.
    Engine(Engine),
    /// GNSS.
    GNSS(Gnss),
    /// Host.
    Host(Host),
    /// Motion.
    Motion(Motion),
    /// Target.
    Target(Target),
    /// Encoder.
    Encoder((u8, f32)), // TODO: There could be a struct for this somewhere
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectType {
    Command,
    Signal,
}

pub struct ObjectMessage {
    /// Object.
    pub object: Object,
    /// Object type.
    pub object_type: ObjectType,
    /// Timestamp of queueing.
    pub timestamp: std::time::Instant,
}

impl ObjectMessage {
    /// Create a new command message.
    pub fn command(object: Object) -> Self {
        Self {
            object,
            object_type: ObjectType::Command,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Create a new signal message.
    pub fn signal(object: Object) -> Self {
        Self {
            object,
            object_type: ObjectType::Signal,
            timestamp: std::time::Instant::now(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde_derive::Deserialize)]
pub enum MachineType {
    /// Excavator.
    Excavator = 1,
    /// Wheel loader.
    WheelLoader = 2,
    /// Dozer.
    Dozer = 3,
    /// Grader.
    Grader = 4,
    /// Hauler.
    Hauler = 5,
    /// Forestry.
    Forestry = 6,
}

impl TryFrom<u8> for MachineType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Excavator),
            2 => Ok(Self::WheelLoader),
            3 => Ok(Self::Dozer),
            4 => Ok(Self::Grader),
            5 => Ok(Self::Hauler),
            6 => Ok(Self::Forestry),
            _ => Err(()),
        }
    }
}
