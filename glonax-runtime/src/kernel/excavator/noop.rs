use crate::runtime::program::*;

use super::HydraulicMotion;

pub(super) struct NoopProgram {
    domain: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>,
}

impl NoopProgram {
    pub fn new(model: std::sync::Arc<tokio::sync::RwLock<super::body::Body>>) -> Self {
        Self { domain: model }
    }
}

#[async_trait::async_trait]
impl Program for NoopProgram {
    type MotionPlan = HydraulicMotion;

    /// Propagate the program forwards.
    ///
    /// This method returns an optional motion instruction.
    async fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan> {
        trace!("Last step: {:?}", context.last_step.elapsed());

        if let Ok(mut domain) = self.domain.try_write() {
            domain.signal_update(context.reader).await;
        }

        if let Ok(domain) = self.domain.try_read() {
            if let Some(angle_slew) = domain.rig().angle_slew() {
                debug!(
                    "Slew angle: {:>+5.2}rad {:>+5.2}°",
                    angle_slew,
                    crate::core::rad_to_deg(angle_slew)
                );
            }
            if let Some(angle_boom) = domain.rig().angle_boom() {
                debug!(
                    "Boom angle: {:>+5.2}rad {:>+5.2}°",
                    angle_boom,
                    crate::core::rad_to_deg(angle_boom)
                );
            }
            if let Some(angle_arm) = domain.rig().angle_arm() {
                debug!(
                    "Arm angle: {:>+5.2}rad {:>+5.2}°",
                    angle_arm,
                    crate::core::rad_to_deg(angle_arm)
                );
            }

            if let Some(boom_point) = domain.boom_point() {
                debug!(
                    "Boom point: X {:>+5.2} Y {:>+5.2}",
                    boom_point.x, boom_point.y,
                );
            }

            if let Some(effector_point_agl) = domain.effector_point_abs() {
                let effector_point = domain.effector_point().unwrap();

                debug!(
                    "Effector point: X {:>+5.2} Y {:>+5.2} Z {:>+5.2}",
                    effector_point.x, effector_point.y, effector_point.z,
                );
                debug!(
                    "Effector point AGL: X {:>+5.2} Y {:>+5.2} Z {:>+5.2}",
                    effector_point_agl.x, effector_point_agl.y, effector_point_agl.z,
                );
            }
        }

        None
    }

    /// Program termination condition.
    ///
    /// Check if program is finished.
    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
