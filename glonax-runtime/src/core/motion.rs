/// Motion instruction.
///
/// Whether or not the instruction has positive effect
/// depends on the motion device itself. The motion device
/// may support more or less functionality to control motion.
///
/// The motion value can communicate the full range of an i16.
/// The signness of the value is often used as a forward/backward
/// motion indicator. However this is left to the motion device.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Motion {
    /// Stop all motion until resumed.
    StopAll,
    /// Resume all motion.
    ResumeAll,
    /// Stop motion on actuators.
    Stop(Vec<u32>),
    /// Change motion on actuators.
    Change(Vec<(u32, i16)>),
}

pub trait ToMotion: Sync + Send {
    fn to_motion(self) -> Motion;
}

impl ToMotion for Motion {
    fn to_motion(self) -> Motion {
        self
    }
}

impl std::fmt::Display for Motion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Motion::StopAll => write!(f, "Stop all"),
            Motion::ResumeAll => write!(f, "Resume all"),

            Motion::Stop(actuators) => {
                write!(
                    f,
                    "Stop: {}",
                    actuators
                        .iter()
                        .map(|f| format!("Actuator: {}, ", f))
                        .collect::<String>()
                )
            }
            Motion::Change(actuators) => {
                write!(
                    f,
                    "Change: {}",
                    actuators
                        .iter()
                        .map(|f| format!("Actuator: {}; Value: {}, ", f.0, f.1))
                        .collect::<String>()
                )
            }
        }
    }
}
