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
    Control(Control),
    Engine(Engine),
    GNSS(Gnss),
    Host(Host),
    Motion(Motion),
    Target(Target),
}

// NOTE: Return hash based on the variant of the enum can have unintended consequences.
impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Object::Control(_) => 0.hash(state),
            Object::Engine(_) => 2.hash(state),
            Object::GNSS(_) => 3.hash(state),
            Object::Host(_) => 4.hash(state),
            Object::Motion(_) => 5.hash(state),
            Object::Target(_) => 6.hash(state),
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
