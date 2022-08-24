use crate::core::motion::{Motion, ToMotion};

#[repr(C)]
pub enum MotionType {
    /// Stop all motion until resumed.
    StopAll = 1,
    /// Resume all motion.
    ResumeAll = 2,
    /// Stop motion on actuators.
    Stop = 3,
    /// Change motion on actuators.
    Change = 4,
}

#[repr(C)]
pub struct SchematicMotion {
    actuator: [u32; 8],
    value: [i16; 8],
    ty: MotionType,
}

impl SchematicMotion {
    pub fn from_motion<T: ToMotion>(motion: T) -> Self {
        let motion = motion.to_motion();

        let mut xx = Self {
            actuator: [0xffff; 8],
            value: [0xff; 8],
            ty: MotionType::StopAll,
        };

        xx.ty = match motion {
            Motion::StopAll => MotionType::StopAll,
            Motion::ResumeAll => MotionType::ResumeAll,
            Motion::Stop(v) => {
                for (idx, ele) in v.as_slice().iter().enumerate() {
                    xx.actuator[idx] = *ele;
                }
                MotionType::Stop
            }
            Motion::Change(v) => {
                for (idx, ele) in v.as_slice().iter().enumerate() {
                    xx.actuator[idx] = (*ele).0;
                    xx.value[idx] = (*ele).1;
                }
                MotionType::Change
            }
        };

        xx
    }
}

impl AsRef<[u8]> for SchematicMotion {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl TryFrom<&[u8]> for SchematicMotion {
    type Error = (); // TODO: Error.

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(unsafe { std::ptr::read(value.as_ptr() as *const _) })
    }
}

impl ToMotion for SchematicMotion {
    fn to_motion(self) -> crate::core::motion::Motion {
        match self.ty {
            MotionType::StopAll => crate::core::motion::Motion::StopAll,
            MotionType::ResumeAll => crate::core::motion::Motion::ResumeAll,
            MotionType::Stop => crate::core::motion::Motion::Stop(
                self.actuator
                    .iter()
                    .filter(|actuator| actuator != &&0xffff)
                    .map(|actuator| *actuator)
                    .collect(),
            ),
            MotionType::Change => crate::core::motion::Motion::Change(
                self.actuator
                    .iter()
                    .filter(|actuator| actuator != &&0xffff)
                    .zip(self.value)
                    .map(|(actuator, value)| (*actuator, value))
                    .collect(),
            ),
        }
    }
}
