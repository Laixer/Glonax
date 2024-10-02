use std::time::Instant;

pub use self::control::Control;
pub use self::engine::{Engine, EngineState};
pub use self::gnss::{Gnss, GnssStatus};
pub use self::instance::Instance;
pub use self::motion::Actuator;
pub use self::motion::Motion;
pub use self::rotation::{RotationReference, Rotator};
pub use self::status::{ModuleError, ModuleState, ModuleStatus};
pub use self::target::Target;

mod control;
mod engine;
mod gnss;
mod instance;
mod motion;
mod rotation;
mod status;
mod target;

/// Represents an object in the system.
#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    /// Control.
    Control(Control),
    /// Engine.
    Engine(Engine),
    // /// GNSS.
    // GNSS(Gnss),
    /// Motion.
    Motion(Motion),
    /// Target.
    Target(Target),
    /// Rotator.
    Rotator(Rotator),
    /// Module status.
    ModuleStatus(ModuleStatus),
}

/// Represents the type of an object.
#[derive(Clone, Debug, PartialEq)]
pub enum ObjectType {
    /// Command object.
    Command,
    /// Signal object.
    Signal,
}

/// Represents a message containing an object.
#[derive(Clone, Debug)]
pub struct ObjectMessage {
    /// The object associated with the message.
    pub object: Object,
    /// The type of the object.
    pub object_type: ObjectType,
    /// The timestamp of when the message was queued.
    pub timestamp: Instant,
}

impl ObjectMessage {
    /// Create a new command message.
    ///
    /// # Arguments
    ///
    /// * `object` - The object associated with the message.
    ///
    /// # Returns
    ///
    /// A new `ObjectMessage` with the specified object and message type set to `Command`.
    pub fn command(object: Object) -> Self {
        Self {
            object,
            object_type: ObjectType::Command,
            timestamp: Instant::now(),
        }
    }

    /// Create a new signal message.
    ///
    /// # Arguments
    ///
    /// * `object` - The object associated with the message.
    ///
    /// # Returns
    ///
    /// A new `ObjectMessage` with the specified object and message type set to `Signal`.
    pub fn signal(object: Object) -> Self {
        Self {
            object,
            object_type: ObjectType::Signal,
            timestamp: Instant::now(),
        }
    }
}

/// Represents the type of a machine.
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

/// Converts a u8 value into a `MachineType` enum variant.
///
/// # Arguments
///
/// * `value` - The u8 value to convert.
///
/// # Returns
///
/// Returns a `Result` containing the converted `MachineType` variant if the conversion is successful,
/// or an `Err` value if the conversion fails.
///
/// # Examples
///
/// ```rust
/// use glonax::core::MachineType;
///
/// let value: u8 = 1;
/// let machine_type = MachineType::try_from(value);
/// assert_eq!(machine_type, Ok(MachineType::Excavator));
/// ```
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
