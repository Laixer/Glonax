use glonax_core::{algorithm::lowpass::SimpleExpSmoothing, metric::MetricValue, motion::Motion};

use crate::runtime::{operand::*, Signal};

use super::Actuator;

// const SET_POINT: f32 = -std::f32::consts::PI / 2.0;
const SET_POINT: f32 = -1.0;
const EXP_ALPHA: f32 = 0.1;
const PROP_FACTOR: f32 = 70.0;

pub struct ArmBalanceProgram {
    angle: Option<f32>,
    filter: SimpleExpSmoothing,
}

impl ArmBalanceProgram {
    pub fn new() -> Self {
        Self {
            angle: None,
            filter: SimpleExpSmoothing::new(EXP_ALPHA),
        }
    }
}

impl Program for ArmBalanceProgram {
    fn can_terminate(&self, _: &mut Context) -> bool {
        self.angle
            .map_or(false, |angle| (SET_POINT - angle).abs() < 0.02)
    }

    fn term_action(&self, _: &mut Context) -> Option<Motion> {
        Some(Motion::Stop(vec![Actuator::Arm.into()]))
    }

    fn push(&mut self, domain: Signal) {
        match domain.value {
            MetricValue::Acceleration(vec) => {
                let signal_angle = -vec.x.atan2(vec.y);

                let forecast_angle = self.filter.fit(signal_angle);

                debug!(
                    "Angle {:>+5.2} Forecast {:>+5.2}",
                    signal_angle, forecast_angle
                );

                self.angle = Some(forecast_angle);
            }
            _ => self.angle = None,
        }
    }

    fn step(&mut self, _context: &mut Context) -> Option<Motion> {
        self.angle.map_or(None, |angle| {
            let err = SET_POINT - angle;
            let power = ((err.abs() * PROP_FACTOR) + 175.0).min(255.0);

            let power = if err.is_sign_negative() {
                -power
            } else {
                power
            };

            debug!(
                "Angle {:>+5.2} Error {:>+5.2} Power {:>+5}",
                angle,
                err,
                power.round()
            );

            Some(Motion::Change(vec![(
                Actuator::Arm.into(),
                power.round() as i16,
            )]))
        })
    }
}
