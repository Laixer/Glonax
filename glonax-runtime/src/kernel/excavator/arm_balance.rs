use glonax_core::{
    metric::MetricValue,
    motion::{Motion, NormalControl},
    position::Position,
};

use crate::runtime::operand::*;

use super::Actuator;

// TODO: Find range.
// Arm range: -0.45 <> -2.47 (25 <> 140)
// Boom range:
// Bucket range:

const SET_POINT: f32 = -std::f32::consts::PI / 2.;
const KP: f32 = 2.7;
const KI: f32 = 0.0;
const KD: f32 = 0.4;

pub struct ArmBalanceProgram {
    pid: pid::Pid<f32>,
    pos: Option<Position>,
    diff: f32,
    iteration: u32,
}

impl ArmBalanceProgram {
    pub fn new() -> Self {
        Self {
            pid: pid::Pid::new(KP, KI, KD, 1.0, 1.0, 1.0, 1.0, SET_POINT),
            pos: None,
            diff: 0.,
            iteration: 0,
        }
    }
}

impl Program for ArmBalanceProgram {
    fn can_terminate(&self, _: &mut Context) -> bool {
        if let Some(pos) = self.pos {
            let e = SET_POINT - pos.pitch;
            e.abs() < 0.1
        } else {
            false
        }
    }

    fn term_action(&self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Stop(vec![Actuator::Arm.into()])) // TODO: auto conv.
    }

    fn push(&mut self, _: u32, value: MetricValue, _: &mut Context) {
        match value {
            // MetricValue::Position(pos) => match id {
            //     0 => {
            //         if let Some(lpos) = self.pos {
            //             self.diff = lpos.pitch - pos.pitch;
            //         }
            //         self.pos = Some(pos);
            //     }
            //     _ => {}
            // },
            _ => {}
        }
    }

    fn step(&mut self, _: &mut Context) -> Option<Motion> {
        self.iteration += 1;

        if self.pos.is_none() {
            return None;
        }

        if self.diff < -0.2 {
            return None;
        }

        debug!(
            "{} Position: Pitch {:+.5} Delta {:+.5} Error {:+.5}",
            self.iteration,
            self.pos.unwrap().pitch,
            self.diff,
            SET_POINT - self.pos.unwrap().pitch,
        );
        let output = self.pid.next_control_output(self.pos.unwrap().pitch);
        debug!("Output: {}", output.output);

        Some(
            NormalControl {
                actuator: Actuator::Arm.into(),
                value: output.output,
                ..Default::default()
            }
            .into(),
        )
    }
}
