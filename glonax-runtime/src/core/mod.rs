pub use self::control::Control;
pub use self::engine::{Engine, EngineRequest, EngineState, EngineStatus};
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Object {
    Engine = 1,
    Gnss = 2,
    Host = 3,
    Instance = 4,
    Motion = 5,
    Target = 6,
    Status = 7,
}

impl TryFrom<u8> for Object {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Engine),
            2 => Ok(Self::Gnss),
            3 => Ok(Self::Host),
            4 => Ok(Self::Instance),
            5 => Ok(Self::Motion),
            6 => Ok(Self::Target),
            7 => Ok(Self::Status),
            _ => Err(()),
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
