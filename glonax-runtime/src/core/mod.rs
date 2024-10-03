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

/// Represents a message associated with an object in the system.
///
/// This struct contains information about the object, its type, and the timestamp
/// indicating when the message was queued.
///
/// # Fields
///
/// * `object` - The object associated with the message.
/// * `object_type` - The type of the object.
/// * `timestamp` - The timestamp of when the message was queued.
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

#[repr(C)]
pub struct Repository {
    /// Instance.
    instance: Instance,
    /// Machine type.
    machine_type: MachineType,
    /// Engine.
    engine: Engine,
    /// Control.
    control: Vec<Control>,
    /// Rotator.
    rotator: Vec<Rotator>,
    /// Module status.
    module_status: Vec<ModuleStatus>,
}

impl Repository {
    pub fn new(
        instance: Instance,
        machine_type: MachineType,
        engine: Engine,
        control: Vec<Control>,
        rotator: Vec<Rotator>,
        module_status: Vec<ModuleStatus>,
    ) -> Self {
        Self {
            instance,
            machine_type,
            engine,
            control,
            rotator,
            module_status,
        }
    }

    #[inline]
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    #[inline]
    pub fn machine_type(&self) -> MachineType {
        self.machine_type
    }

    #[inline]
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    #[inline]
    pub fn control(&self) -> &[Control] {
        &self.control
    }

    #[inline]
    pub fn rotator(&self) -> &[Rotator] {
        &self.rotator
    }

    #[inline]
    pub fn module_status(&self) -> &[ModuleStatus] {
        &self.module_status
    }
}
