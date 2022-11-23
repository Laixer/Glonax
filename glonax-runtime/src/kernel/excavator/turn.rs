use crate::runtime::operand::*;

use super::HydraulicMotion;

pub(super) struct TurnProgram {
    domain: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
    objective: super::body::Objective,
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
        }
    }
}

#[async_trait::async_trait]
impl Program for TurnProgram {
    type MotionPlan = HydraulicMotion;

    /// Propagate the program forwards.
    ///
    /// This method returns an optional motion instruction.
    async fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        if let Ok(mut domain) = self.domain.try_write() {
            domain.signal_update(&mut context.reader).await;
        }

        let rig_error = self.objective.erorr_diff();

        let mut motion_vector = vec![];

        if let Some(error) = rig_error.angle_slew() {
            // Optimal error path.
            let error = if error.abs() > std::f32::consts::PI {
                error - (2.0 * std::f32::consts::PI)
            } else {
                error
            };

            let power = super::consts::MOTION_PROFILE_SLEW.proportional_power(error);
            motion_vector.push((super::Actuator::Slew, power));
        }

        if !motion_vector.is_empty() {
            Some(HydraulicMotion::Change(motion_vector))
        } else {
            None
        }
    }

    /// Program termination condition.
    ///
    /// Check if program is finished.
    fn can_terminate(&self, _context: &mut Context) -> bool {
        let rig_error = self.objective.erorr_diff();

        if let Some(error) = rig_error.angle_slew() {
            error < 0.02
        } else {
            false
        }
    }

    /// Program termination action.
    ///
    /// This is an optional method to send a last motion
    /// instruction. This method is called after `can_terminate`
    /// returns true and before the program is terminated.
    fn term_action(&self, _context: &mut Context) -> Option<Self::MotionPlan> {
        Some(HydraulicMotion::Stop(vec![super::Actuator::Slew]))
    }
}
