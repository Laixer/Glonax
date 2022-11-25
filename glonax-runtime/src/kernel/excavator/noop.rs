use crate::runtime::operand::*;

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
            domain.signal_update(&mut context.reader).await;
        }

        if let Ok((source, signal)) = context.reader.recv().await {
            debug!("Source {} ⇨ {}", source, signal.value);
        }

        if let Ok(domain) = self.domain.try_read() {
            if let Some(effector_point_agl) = domain.effector_point_abs() {
                let boom_point = domain.boom_point().unwrap();
                let effector_point = domain.effector_point().unwrap();

                debug!(
                    "Boom point: X {:>+5.2} Y {:>+5.2}",
                    boom_point.x, boom_point.y,
                );
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
