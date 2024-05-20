use crate::{
    runtime::{CommandSender, Component, ComponentContext},
    MachineState,
};

struct Governor {
    /// Default engine speed.
    rpm_idle: u16,
    /// Maximum RPM for the engine.
    rpm_max: u16,
    /// Engine state transition timeout.
    state_transition_timeout: std::time::Duration,
}

// TODO: Remove this when we have a proper implementation.
impl std::default::Default for Governor {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Governor {
    /// Construct a new governor.
    fn new(rpm_idle: u16, rpm_max: u16) -> Self {
        Self {
            rpm_idle,
            rpm_max,
            state_transition_timeout: std::time::Duration::from_millis(2_000),
        }
    }

    /// Reshape the torque.
    ///
    /// This method reshapes the torque based on the engine speed.
    #[inline]
    fn reshape(&self, torque: u16) -> u16 {
        torque.clamp(self.rpm_idle, self.rpm_max)
    }

    /// Get the next engine state.
    ///
    /// This method determines the next engine state based on the actual and requested
    /// engine states. It returns the next engine state as an `EngineRequest`.
    fn next_state(
        &self,
        signal: &crate::core::Engine,
        command: &crate::core::Engine,
        command_instant: Option<std::time::Instant>,
    ) -> crate::core::Engine {
        use crate::core::EngineState;

        match (signal.state, command.state) {
            (EngineState::NoRequest, EngineState::Starting) => {
                if let Some(instant) = command_instant {
                    if instant.elapsed() > self.state_transition_timeout {
                        return crate::core::Engine {
                            rpm: self.reshape(self.rpm_idle),
                            state: EngineState::NoRequest,
                            ..Default::default()
                        };
                    }
                }

                crate::core::Engine {
                    rpm: self.reshape(self.rpm_idle),
                    state: EngineState::Starting,
                    ..Default::default()
                }
            }
            (EngineState::NoRequest, EngineState::Request) => {
                if let Some(instant) = command_instant {
                    if instant.elapsed() > self.state_transition_timeout {
                        return crate::core::Engine {
                            rpm: self.reshape(self.rpm_idle),
                            state: EngineState::NoRequest,
                            ..Default::default()
                        };
                    }
                }

                crate::core::Engine {
                    rpm: self.reshape(self.rpm_idle),
                    state: EngineState::Starting,
                    ..Default::default()
                }
            }
            (EngineState::NoRequest, _) => crate::core::Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::NoRequest,
                ..Default::default()
            },

            (EngineState::Starting, _) => {
                if let Some(instant) = command_instant {
                    if instant.elapsed() > self.state_transition_timeout {
                        return crate::core::Engine {
                            rpm: self.reshape(self.rpm_idle),
                            state: EngineState::NoRequest,
                            ..Default::default()
                        };
                    }
                }

                crate::core::Engine {
                    rpm: self.reshape(self.rpm_idle),
                    state: EngineState::Starting,
                    ..Default::default()
                }
            }
            (EngineState::Stopping, _) => crate::core::Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::Stopping,
                ..Default::default()
            },

            (EngineState::Request, EngineState::NoRequest) => crate::core::Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::Stopping,
                ..Default::default()
            },
            (EngineState::Request, EngineState::Starting) => crate::core::Engine {
                rpm: self.reshape(command.rpm),
                state: EngineState::Request,
                ..Default::default()
            },
            (EngineState::Request, EngineState::Stopping) => crate::core::Engine {
                rpm: self.reshape(self.rpm_idle),
                state: EngineState::Stopping,
                ..Default::default()
            },
            (EngineState::Request, EngineState::Request) => crate::core::Engine {
                rpm: self.reshape(command.rpm),
                state: EngineState::Request,
                ..Default::default()
            },
        }
    }
}

pub struct EngineComponent {
    governor: Governor,
}

impl<Cnf: Clone> Component<Cnf> for EngineComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {
            governor: Governor::new(800, 2_100),
        }
    }

    fn tick(
        &mut self,
        _ctx: &mut ComponentContext,
        _state: &mut MachineState,
        command_tx: CommandSender,
    ) {
        let engine_signal = crate::core::Engine {
            rpm: 0,
            state: crate::core::EngineState::NoRequest,
            driver_demand: 0,
            actual_engine: 0,
        };
        let engine_command = Some(crate::core::Engine {
            rpm: 0,
            state: crate::core::EngineState::NoRequest,
            driver_demand: 0,
            actual_engine: 0,
        });
        let engine_command_instant = None;

        //

        let mut engine_command = engine_command.unwrap_or(engine_signal);
        engine_command.actual_engine = 0;
        engine_command.state = match engine_command.state {
            crate::core::EngineState::NoRequest => crate::core::EngineState::NoRequest,
            crate::core::EngineState::Request => crate::core::EngineState::Request,
            _ => engine_signal.state,
        };

        engine_command.driver_demand = engine_command.driver_demand.clamp(0, 100);

        if engine_command.rpm == 0 {
            if engine_command.driver_demand == 0 {
                engine_command.state = crate::core::EngineState::NoRequest;
            } else {
                engine_command.rpm = (engine_command.driver_demand as f32 / 100.0
                    * self.governor.rpm_max as f32) as u16;
            }
        } else {
            engine_command.state = crate::core::EngineState::Request;
        }

        let governor_engine =
            self.governor
                .next_state(&engine_signal, &engine_command, engine_command_instant);

        log::trace!("Engine governor: {:?}", governor_engine);

        // if let Err(e) = command_tx.try_send(crate::core::Object::Engine(governor_engine)) {
        //     log::error!("Failed to send engine command: {}", e);
        // }
    }
}
