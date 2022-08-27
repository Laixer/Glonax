use crate::runtime::operand::*;

use super::{consts::DRIVE_SPEED_MAX, Actuator, HydraulicMotion};

/// Drive strait forward.
///
/// This program is part of the excavator kernel. It drives both tracks straight
/// forward until the desired position is reached. It takes acceleration and
/// decceleration into account.
pub(super) struct DriveProgram {
    profile: TrapezoidalDistanceProfile,
}

struct TrapezoidalProfile {
    ramp_time: std::time::Duration,
    motion_time: std::time::Duration,
    power_range: (f32, f32),
}

impl TrapezoidalProfile {
    fn phase_frame(&self, duration: &std::time::Duration) -> (i32, f32) {
        let p_delta = (self.power_range.1 - self.power_range.0) / self.ramp_time.as_millis() as f32;

        if duration < &self.ramp_time {
            let power = self.power_range.0 + duration.as_millis() as f32 * p_delta;

            (0, power)
        } else if duration > &self.ramp_time && duration < &self.motion_time {
            (1, self.power_range.1)
        } else {
            if let Some(duration_phase2) =
                duration.checked_sub(self.motion_time.max(self.ramp_time))
            {
                if duration_phase2 < self.ramp_time {
                    let power = self.power_range.1 - duration_phase2.as_millis() as f32 * p_delta;
                    (2, power)
                } else {
                    (2, 0.0)
                }
            } else {
                (2, 0.0)
            }
        }
    }

    fn is_finished(&self, duration: &std::time::Duration) -> bool {
        let (phase, power) = self.phase_frame(duration);
        phase == 2 && power == 0.0
    }
}

struct TrapezoidalDistanceProfile {
    inner: TrapezoidalProfile,
    max_speed: f32,
}

impl TrapezoidalDistanceProfile {
    fn new(max_speed: f32, distance: f32) -> Self {
        Self {
            inner: TrapezoidalProfile {
                ramp_time: std::time::Duration::from_secs(3),
                motion_time: std::time::Duration::from_secs_f32(distance / max_speed),
                power_range: (175.0, 255.0),
            },
            max_speed,
        }
    }

    fn phase_frame(&self, duration: &std::time::Duration) -> (i32, f32, f32) {
        let (phase, power) = self.inner.phase_frame(duration);

        let distance = match phase {
            0 => {
                ((self.max_speed * self.inner.ramp_time.as_secs_f32()).sqrt()
                    / self.inner.ramp_time.as_secs_f32())
                    * duration.as_secs_f32()
            }
            1 => {
                (self.max_speed * self.inner.ramp_time.as_secs_f32()).sqrt()
                    + self.max_speed * (duration.as_secs_f32() - self.inner.ramp_time.as_secs_f32())
            }
            2 => {
                (self.max_speed * self.inner.ramp_time.as_secs_f32()).sqrt()
                    + self.max_speed
                        * (self
                            .inner
                            .motion_time
                            .max(self.inner.ramp_time)
                            .as_secs_f32()
                            - self.inner.ramp_time.as_secs_f32())
                    + ((self.max_speed * self.inner.ramp_time.as_secs_f32()).sqrt()
                        / self.inner.ramp_time.as_secs_f32())
                        * duration
                            .checked_sub(self.inner.motion_time.max(self.inner.ramp_time))
                            .unwrap()
                            .as_secs_f32()
            }
            _ => 0.0,
        };

        (phase, power, distance)
    }

    fn is_finished(&self, duration: &std::time::Duration) -> bool {
        self.inner.is_finished(duration)
    }
}

impl DriveProgram {
    pub fn new(params: Parameter) -> Self {
        if params.len() != 1 {
            panic!("Expected 1 parameter, got {}", params.len());
        } else if params[0] == 0.0 {
            panic!("Distance cannot be zero");
        }

        Self {
            profile: TrapezoidalDistanceProfile::new(DRIVE_SPEED_MAX, params[0]),
        }
    }
}

#[async_trait::async_trait]
impl Program for DriveProgram {
    type MotionPlan = HydraulicMotion;

    async fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        let (phase, power, distance) = self.profile.phase_frame(&context.start.elapsed());

        debug!(
            "Phase {}; Time: {}; Power: {}; Distance: {}",
            phase,
            context.start.elapsed().as_secs(),
            power.round(),
            distance.round()
        );

        Some(HydraulicMotion::StraightDrive(power.round() as i16))
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        self.profile.is_finished(&context.start.elapsed())
    }

    fn term_action(&self, _: &mut Context) -> Option<Self::MotionPlan> {
        Some(HydraulicMotion::Stop(vec![
            Actuator::LimpLeft,
            Actuator::LimpRight,
        ]))
    }
}
