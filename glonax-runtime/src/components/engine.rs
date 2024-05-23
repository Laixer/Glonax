use crate::{
    core::{Engine, Object},
    runtime::{CommandSender, ComponentContext, PostComponent, SignalSender},
};

// TODO: Move to drivers?
struct Governor {
    /// Default engine speed.
    rpm_idle: u16,
    /// Maximum RPM for the engine.
    rpm_max: u16,
    /// Engine state transition timeout.
    state_transition_timeout: std::time::Duration,
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

        // command.rpm = (command.driver_demand as f32 / 100.0 * self.rpm_max as f32) as u16;
        // driver_demand: command.driver_demand.clamp(0, 100),

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

impl<Cnf: Clone> PostComponent<Cnf> for EngineComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {
            governor: Governor::new(800, 2_100),
        }
    }

    fn finalize(
        &self,
        ctx: &mut ComponentContext,
        command_tx: CommandSender,
        signal_tx: std::rc::Rc<SignalSender>,
    ) {
        if ctx.machine.emergency {
            if let Err(e) = command_tx.try_send(Object::Engine(Engine::shutdown())) {
                log::error!("Failed to send engine command: {}", e);
            }

            return;
        }

        let engine_signal = ctx.machine.engine_signal;
        let governor_engine = self.governor.next_state(
            &engine_signal,
            &ctx.machine.engine_command.unwrap_or(engine_signal),
            ctx.machine.engine_command_instant,
        );

        log::trace!("Engine governor: {:?}", governor_engine);

        if let Err(e) = command_tx.try_send(Object::Engine(governor_engine)) {
            log::error!("Failed to send engine command: {}", e);
        }

        if ctx.machine.engine_signal_set {
            if let Err(e) = signal_tx.send(Object::Engine(ctx.machine.engine_signal)) {
                log::error!("Failed to send engine signal: {}", e);
            }
        }
        if ctx.machine.vms_signal_set {
            if let Err(e) = signal_tx.send(Object::Host(ctx.machine.vms_signal)) {
                log::error!("Failed to send host signal: {}", e);
            }
        }
        if ctx.machine.gnss_signal_set {
            if let Err(e) = signal_tx.send(Object::GNSS(ctx.machine.gnss_signal)) {
                log::error!("Failed to send gnss signal: {}", e);
            }
        }
    }
}
