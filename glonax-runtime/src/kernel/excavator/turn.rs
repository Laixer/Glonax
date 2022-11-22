use std::time::Duration;

use crate::runtime::operand::*;

use super::HydraulicMotion;

/// Turn strait forward.
///
/// This program is part of the excavator kernel. It drives both tracks then
/// stops when the time limit is reached.
pub(super) struct TurnProgram {
    domain: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
    objective: super::body::Objective,
    _drive_time: Duration,
}

impl TurnProgram {
    pub fn new(
        model: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
        params: Parameter,
    ) -> Self {
        if params.len() != 1 {
            panic!("Expected 1 parameter, got {}", params.len());
        } else if params[0] == 0.0 {
            panic!("Duration cannot be zero");
        }

        Self {
            domain: model.clone(),
            objective: super::body::Objective::new(
                model,
                super::body::Rig {
                    angle_slew: Some(1.57),
                    angle_boom: None,
                    angle_arm: None,
                    angle_attachment: None,
                },
            ),
            _drive_time: Duration::from_secs(params[0] as u64),
        }
    }
}

#[async_trait::async_trait]
impl Program for TurnProgram {
    type MotionPlan = HydraulicMotion;

    async fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        if let Ok(mut domain) = self.domain.try_write() {
            domain.signal_update(&mut context.reader).await;
        }

        let rig_error = self.objective.erorr_diff();

        let mut motion_vector = vec![];

        if let Some(error) = rig_error.angle_slew() {
            let arm_profile = super::body::MotionProfile {
                scale: 10_000.0,
                offset: 10_000,
                limit: 20_000,
                cutoff: 0.02,
            };

            if let Some(power_slew) = arm_profile.proportional_power(error) {
                motion_vector.push((super::Actuator::Slew, power_slew));
            }
        }

        if !motion_vector.is_empty() {
            Some(HydraulicMotion::Change(motion_vector))
        } else {
            None
        }
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        let rig_error = self.objective.erorr_diff();

        if let Some(error) = rig_error.angle_slew() {
            error < 0.02
        } else {
            false
        }

        // context.start.elapsed() >= self.drive_time
    }
}
