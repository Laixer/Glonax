use std::time::{Duration, Instant};

use crate::core::{Engine, EngineState};

#[derive(Clone, Copy)]
pub struct Governor {
    /// Default engine speed.
    rpm_idle: u16,
    /// Maximum RPM for the engine.
    rpm_max: u16,
    /// Engine state transition timeout.
    state_transition_timeout: Duration,
}

impl Governor {
    /// Construct a new governor.
    ///
    /// # Arguments
    ///
    /// * `rpm_idle` - The idle RPM value.
    /// * `rpm_max` - The maximum RPM value.
    /// * `state_transition_timeout` - The timeout duration for state transitions.
    ///
    /// # Returns
    ///
    /// A new `Governor` instance.
    pub fn new(rpm_idle: u16, rpm_max: u16, state_transition_timeout: Duration) -> Self {
        Self {
            rpm_idle,
            rpm_max,
            state_transition_timeout,
        }
    }

    /// Reshape the torque.
    ///
    /// This method reshapes the torque based on the engine speed.
    ///
    /// # Arguments
    ///
    /// * `torque` - The torque value to reshape.
    ///
    /// # Returns
    ///
    /// The reshaped torque value.
    #[inline]
    pub fn reshape(&self, torque: u16) -> u16 {
        torque.clamp(self.rpm_idle, self.rpm_max)
    }

    /// Get the next engine state.
    ///
    /// This method determines the next engine state based on the actual and requested
    /// engine states. It returns the next engine state as an `EngineRequest`.
    ///
    /// # Arguments
    ///
    /// * `signal` - The current engine state.
    /// * `command` - The requested engine state.
    /// * `command_instant` - The instant when the command was issued (optional).
    ///
    /// # Returns
    ///
    /// The next engine state as an `Engine` instance.
    pub fn next_state(
        &self,
        signal: &Engine,
        command: &Engine,
        command_instant: Option<Instant>,
    ) -> Engine {
        // command.rpm = (command.driver_demand as f32 / 100.0 * self.rpm_max as f32) as u16;
        // driver_demand: command.driver_demand.clamp(0, 100),

        match (signal.state, command.state) {
            (EngineState::NoRequest, EngineState::Starting) => {
                if let Some(instant) = command_instant {
                    if instant.elapsed() > self.state_transition_timeout {
                        return Engine {
                            rpm: self.reshape(self.rpm_idle),
                            state: EngineState::NoRequest,
                            ..Default::default()
                        };
                    }
                }

                Engine {
                    rpm: self.reshape(self.rpm_idle),
                    state: EngineState::Starting,
                    ..Default::default()
                }
            }
            (EngineState::NoRequest, EngineState::Request) => {
                if let Some(instant) = command_instant {
                    if instant.elapsed() > self.state_transition_timeout {
                        return Engine {
                            rpm: self.reshape(self.rpm_idle),
                            state: EngineState::NoRequest,
                            ..Default::default()
                        };
                    }
                }

                Engine {
                    rpm: self.reshape(self.rpm_idle),
                    state: EngineState::Starting,
                    ..Default::default()
                }
            }
            (EngineState::NoRequest, _) => Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::NoRequest,
                ..Default::default()
            },

            (EngineState::Starting, _) => {
                if let Some(instant) = command_instant {
                    if instant.elapsed() > self.state_transition_timeout {
                        return Engine {
                            rpm: self.reshape(self.rpm_idle),
                            state: EngineState::NoRequest,
                            ..Default::default()
                        };
                    }
                }

                Engine {
                    rpm: self.reshape(self.rpm_idle),
                    state: EngineState::Starting,
                    ..Default::default()
                }
            }
            (EngineState::Stopping, _) => Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::Stopping,
                ..Default::default()
            },

            (EngineState::Request, EngineState::NoRequest) => Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::Stopping,
                ..Default::default()
            },
            (EngineState::Request, EngineState::Starting) => Engine {
                rpm: self.reshape(command.rpm),
                state: EngineState::Request,
                ..Default::default()
            },
            (EngineState::Request, EngineState::Stopping) => Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::Stopping,
                ..Default::default()
            },
            (EngineState::Request, EngineState::Request) => Engine {
                rpm: self.reshape(command.rpm),
                state: EngineState::Request,
                ..Default::default()
            },
        }
    }
}
