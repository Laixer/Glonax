use crate::{
    core::{Actuator, Motion},
    math::Linear,
};

pub struct ActuatorMotionEvent {
    pub actuator: Actuator,
    pub error: f32,
    pub value: i16,
}

pub struct ActuatorState {
    profile: Linear,
    actuator: Actuator,
    stop: bool,
}

impl ActuatorState {
    pub fn bind(actuator: Actuator, profile: Linear) -> Self {
        Self {
            profile,
            actuator,
            stop: false,
        }
    }

    pub fn update(&mut self, error: Option<f32>) -> Option<ActuatorMotionEvent> {
        if let Some(error) = error {
            self.stop = false;

            Some(ActuatorMotionEvent {
                actuator: self.actuator,
                error,
                value: self.profile.update(error) as i16,
            })
        } else if !self.stop {
            self.stop = true;

            Some(ActuatorMotionEvent {
                actuator: self.actuator,
                error: 0.0,
                value: Motion::POWER_NEUTRAL,
            })
        } else {
            None
        }
    }
}
