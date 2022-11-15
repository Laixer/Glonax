use std::time::Duration;

use crate::runtime::operand::*;

use super::{Actuator, HydraulicMotion};

const DRIVE_POWER: i16 = -20_096;

/// Turn strait forward.
///
/// This program is part of the excavator kernel. It drives both tracks then
/// stops when the time limit is reached.
pub(super) struct TurnProgram {
    drive_time: Duration,
    first: bool,
}

impl TurnProgram {
    pub fn new(params: Parameter) -> Self {
        if params.len() != 1 {
            panic!("Expected 1 parameter, got {}", params.len());
        } else if params[0] == 0.0 {
            panic!("Duration cannot be zero");
        }

        Self {
            drive_time: Duration::from_secs(params[0] as u64),
            first: true,
        }
    }
}

#[async_trait::async_trait]
impl Program for TurnProgram {
    type MotionPlan = HydraulicMotion;

    /// Boot the program.
    ///
    /// This method is called when the runtime accepted
    /// this progam and started its routine.
    // fn boot(&mut self, _context: &mut Context) -> Option<Self::MotionPlan> {
    //     Some(HydraulicMotion::Change(vec![(Actuator::Slew, -20_000)]))
    // }

    async fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        let mut angle_boom_error: Option<f32> = None;
        let set = 1.57;

        if !self.first {
            if let Ok((source, signal)) = context.reader.recv().await {
                match source {
                    super::BODY_PART_FRAME => {
                        if let crate::core::metric::MetricValue::Angle(value) = signal.value {
                            let encoder = crate::signal::Encoder::new(0.0..85.0, 0.0..6.28);

                            let angle = encoder.scale(value.x as f32);
                            let percentage = encoder.scale_to(100.0, value.x as f32);

                            angle_boom_error = Some(set - angle);

                            debug!(
                                "Turn Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%",
                                value.x,
                                angle,
                                crate::core::rad_to_deg(angle),
                                percentage,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        self.first = false;

        if angle_boom_error.is_some() {
            let power_boom = (angle_boom_error.unwrap() * 10_000.0) as i16;
            let power_boom = if angle_boom_error.unwrap().is_sign_positive() {
                (-power_boom).max(-20_000) - 10_000
            } else {
                (-power_boom).min(20_000) + 10_000
            };

            debug!("power_boom: {:?}", power_boom);

            // tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            if angle_boom_error.unwrap().abs() > 0.02 {
                // motion_vector.push((super::Actuator::Boom, power_boom));

                debug!("steerrr....");

                Some(HydraulicMotion::Change(vec![(Actuator::Slew, power_boom)]))
            } else {
                None
            }
            // Some(HydraulicMotion::Change(vec![(Actuator::Slew, -32_000)]))
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;

            Some(HydraulicMotion::Change(vec![(Actuator::Slew, -20_000)]))
        }
    }

    fn can_terminate(&self, context: &mut Context) -> bool {
        context.start.elapsed() >= self.drive_time
    }

    // fn term_action(&self, _: &mut Context) -> Option<Self::MotionPlan> {
    //     Some(HydraulicMotion::Stop(vec![Actuator::Slew]))
    // }
}
