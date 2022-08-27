use std::{time::Duration, u32};

use super::{Trace, TraceWriter};

/// Motion instruction.
///
/// Whether or not the instruction has positive effect
/// depends on the motion device itself. The motion device
/// may support more or less functionality to control motion.
///
/// The motion value can communicate the full range of an i16.
/// The signness of the value is often used as a forward/backward
/// motion indicator. However this is left to the motion device.
#[derive(Debug)]
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

#[derive(serde::Serialize)]
struct MotionTrace {
    /// Timestamp of the trace.
    timestamp: u128,
    /// Respective actuator.
    actuator: u32,
    /// Motion value.
    value: i16,
}

impl<T: TraceWriter> Trace<T> for Motion {
    fn record(&self, writer: &mut T, timestamp: Duration) {
        match self {
            Motion::StopAll => writer.write_record(MotionTrace {
                timestamp: timestamp.as_millis(),
                actuator: u8::MAX as u32,
                value: 0,
            }),
            // TODO: Maybe something else.
            Motion::ResumeAll => writer.write_record(MotionTrace {
                timestamp: timestamp.as_millis(),
                actuator: u8::MAX as u32,
                value: 0,
            }),
            Motion::Stop(actuators) => {
                for actuator in actuators {
                    writer.write_record(MotionTrace {
                        timestamp: timestamp.as_millis(),
                        actuator: *actuator,
                        value: 0,
                    });
                }
            }
            Motion::Change(actuators) => {
                for (actuator, value) in actuators {
                    writer.write_record(MotionTrace {
                        timestamp: timestamp.as_millis(),
                        actuator: *actuator,
                        value: *value,
                    });
                }
            }
        }
    }
}
