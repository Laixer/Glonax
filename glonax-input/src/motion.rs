use glonax::transport::{Motion, ToMotion};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Actuator {
    /// Boom actuator.
    Boom = 0,
    /// Arm actuator.
    Arm = 4,
    /// Attachment actuator.
    Attachment = 5,
    /// Slew actuator.
    Slew = 1,
    /// Left limp actuator.
    LimpLeft = 3,
    /// Right limp actuator.
    LimpRight = 2,
}

impl From<Actuator> for u32 {
    fn from(value: Actuator) -> Self {
        value as u32
    }
}

pub(crate) enum HydraulicMotion {
    /// Stop all motion until resumed.
    StopAll,
    /// Resume all motion.
    ResumeAll,
    /// Drive straight forward or backwards.
    StraightDrive(i16),
    /// Change motion on actuators.
    Change(Vec<(Actuator, i16)>),
}

#[allow(dead_code)]
impl HydraulicMotion {
    /// Maximum power setting.
    pub(crate) const POWER_MAX: i16 = i16::MAX;
    /// Neutral power setting.
    pub(crate) const POWER_NEUTRAL: i16 = 0;
    /// Minimum power setting.
    pub(crate) const POWER_MIN: i16 = i16::MIN;
}

impl ToMotion for HydraulicMotion {
    fn to_motion(self) -> Motion {
        match self {
            HydraulicMotion::StopAll => glonax::transport::Motion {
                r#type: glonax::transport::motion::MotionType::StopAll.into(),
                changes: vec![],
            },
            HydraulicMotion::ResumeAll => glonax::transport::Motion {
                r#type: glonax::transport::motion::MotionType::ResumeAll.into(),
                changes: vec![],
            },
            HydraulicMotion::StraightDrive(value) => glonax::transport::Motion {
                r#type: glonax::transport::motion::MotionType::Change.into(),
                changes: vec![
                    glonax::transport::motion::ChangeSet {
                        actuator: Actuator::LimpLeft.into(),
                        value: value as i32,
                    },
                    glonax::transport::motion::ChangeSet {
                        actuator: Actuator::LimpRight.into(),
                        value: value as i32,
                    },
                ],
            },
            HydraulicMotion::Change(v) => glonax::transport::Motion {
                r#type: glonax::transport::motion::MotionType::Change.into(),
                changes: v
                    .iter()
                    .map(|(a, va)| glonax::transport::motion::ChangeSet {
                        actuator: (*a).into(),
                        value: *va as i32,
                    })
                    .collect(),
            },
        }
    }
}
