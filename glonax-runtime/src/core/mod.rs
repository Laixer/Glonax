pub use self::engine::Engine;
pub use self::gnss::Gnss;
pub use self::host::Host;
pub use self::instance::Instance;
pub use self::motion::Actuator; // TODO: maybe access via motion::Actuator
pub use self::motion::Motion;
pub use self::pose::Pose;
pub use self::status::Status;
pub use self::target::Target;

mod engine;
mod gnss;
mod host;
mod instance;
mod motion;
mod pose;
mod status;
mod target;

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
