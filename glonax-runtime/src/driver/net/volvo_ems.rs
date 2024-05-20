use j1939::{Frame, FrameBuilder, IdBuilder, PGN};

use crate::{driver::EngineMessage, net::Parsable};

use super::engine::EngineManagementSystem;

// struct Governor {
//     /// Default engine speed.
//     rpm_idle: u16,
//     /// Maximum RPM for the engine.
//     rpm_max: u16,
//     /// Engine state transition timeout.
//     state_transition_timeout: std::time::Duration,
// }

// // TODO: Remove this when we have a proper implementation.
// impl std::default::Default for Governor {
//     fn default() -> Self {
//         Self::new(0, 0)
//     }
// }

// impl Governor {
//     /// Construct a new governor.
//     fn new(rpm_idle: u16, rpm_max: u16) -> Self {
//         Self {
//             rpm_idle,
//             rpm_max,
//             state_transition_timeout: std::time::Duration::from_millis(2_000),
//         }
//     }

//     /// Reshape the torque.
//     ///
//     /// This method reshapes the torque based on the engine speed.
//     #[inline]
//     fn reshape(&self, torque: u16) -> u16 {
//         torque.clamp(self.rpm_idle, self.rpm_max)
//     }

//     /// Get the next engine state.
//     ///
//     /// This method determines the next engine state based on the actual and requested
//     /// engine states. It returns the next engine state as an `EngineRequest`.
//     fn next_state(
//         &self,
//         signal: &core::Engine,
//         command: &core::Engine,
//         command_instant: Option<std::time::Instant>,
//     ) -> crate::core::Engine {
//         use crate::core::EngineState;

//         match (signal.state, command.state) {
//             (EngineState::NoRequest, EngineState::Starting) => {
//                 if let Some(instant) = command_instant {
//                     if instant.elapsed() > self.state_transition_timeout {
//                         return core::Engine {
//                             rpm: self.reshape(self.rpm_idle),
//                             state: EngineState::NoRequest,
//                             ..Default::default()
//                         };
//                     }
//                 }

//                 core::Engine {
//                     rpm: self.reshape(self.rpm_idle),
//                     state: EngineState::Starting,
//                     ..Default::default()
//                 }
//             }
//             (EngineState::NoRequest, EngineState::Request) => {
//                 if let Some(instant) = command_instant {
//                     if instant.elapsed() > self.state_transition_timeout {
//                         return core::Engine {
//                             rpm: self.reshape(self.rpm_idle),
//                             state: EngineState::NoRequest,
//                             ..Default::default()
//                         };
//                     }
//                 }

//                 core::Engine {
//                     rpm: self.reshape(self.rpm_idle),
//                     state: EngineState::Starting,
//                     ..Default::default()
//                 }
//             }
//             (EngineState::NoRequest, _) => core::Engine {
//                 rpm: self.reshape(self.rpm_idle),
//                 state: EngineState::NoRequest,
//                 ..Default::default()
//             },

//             (EngineState::Starting, _) => {
//                 if let Some(instant) = command_instant {
//                     if instant.elapsed() > self.state_transition_timeout {
//                         return core::Engine {
//                             rpm: self.reshape(self.rpm_idle),
//                             state: EngineState::NoRequest,
//                             ..Default::default()
//                         };
//                     }
//                 }

//                 core::Engine {
//                     rpm: self.reshape(self.rpm_idle),
//                     state: EngineState::Starting,
//                     ..Default::default()
//                 }
//             }
//             (EngineState::Stopping, _) => core::Engine {
//                 rpm: self.reshape(self.rpm_idle),
//                 state: EngineState::Stopping,
//                 ..Default::default()
//             },

//             (EngineState::Request, EngineState::NoRequest) => core::Engine {
//                 rpm: self.reshape(self.rpm_idle),
//                 state: EngineState::Stopping,
//                 ..Default::default()
//             },
//             (EngineState::Request, EngineState::Starting) => core::Engine {
//                 rpm: self.reshape(command.rpm),
//                 state: EngineState::Request,
//                 ..Default::default()
//             },
//             (EngineState::Request, EngineState::Stopping) => core::Engine {
//                 rpm: self.reshape(self.rpm_idle),
//                 state: EngineState::Stopping,
//                 ..Default::default()
//             },
//             (EngineState::Request, EngineState::Request) => core::Engine {
//                 rpm: self.reshape(command.rpm),
//                 state: EngineState::Request,
//                 ..Default::default()
//             },
//         }
//     }
// }

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum VolvoEngineState {
    /// Engine shutdown.
    Shutdown = 0b0000_0111,
    /// Engine starter locked.
    Locked = 0b0100_0111,
    /// Engine running at requested speed.
    Nominal = 0b0100_0011,
    /// Engine starter engaged.
    Starting = 0b1100_0011,
}

// #[derive(Copy, Clone, Debug, Default)]
// struct Test {
//     /// Engine command.
//     engine_command: Option<core::Engine>, // INNER SERVICE (engine)
//     /// Engine state request instant.
//     engine_command_instant: Option<std::time::Instant>,
// }

pub struct VolvoD7E {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
    /// Engine management system.
    ems: EngineManagementSystem,
    // /// Engine governor.
    // governor: Governor,
    // /// Some random value.
    // value: std::cell::Cell<Test>,
    // /// Some random value.
    // signal: std::cell::Cell<Test>,
}

impl VolvoD7E {
    /// Construct a new engine management system.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
            ems: EngineManagementSystem::new(da, sa),
            // governor: Governor::new(800, 2_100),
            // value: std::cell::Cell::default(),
            // signal: std::cell::Cell::default(),
        }
    }

    /// Request speed control
    pub fn speed_control(&self, state: VolvoEngineState, rpm: u16) -> Frame {
        FrameBuilder::new(
            IdBuilder::from_pgn(PGN::ProprietaryB(65_282))
                .priority(3)
                .sa(self.source_address)
                .build(),
        )
        .copy_from_slice(&[
            0x00,
            state as u8,
            0x1f,
            0x00,
            0x00,
            0x00,
            0x20,
            (rpm as f32 / 10.0) as u8,
        ])
        .build()
    }

    // #[deprecated]
    // fn governor_mode(&self, engine_signal: core::Engine) -> crate::core::Engine {
    //     let test = self.value.get();

    //     //

    //     let mut engine_command = test.engine_command.unwrap_or(engine_signal);
    //     engine_command.actual_engine = 0;
    //     engine_command.state = match engine_command.state {
    //         core::EngineState::NoRequest => core::EngineState::NoRequest,
    //         core::EngineState::Request => core::EngineState::Request,
    //         _ => engine_signal.state,
    //     };

    //     engine_command.driver_demand = engine_command.driver_demand.clamp(0, 100);

    //     if engine_command.rpm == 0 {
    //         if engine_command.driver_demand == 0 {
    //             engine_command.state = core::EngineState::NoRequest;
    //         } else {
    //             engine_command.rpm = (engine_command.driver_demand as f32 / 100.0
    //                 * self.governor.rpm_max as f32) as u16;
    //         }
    //     } else {
    //         engine_command.state = core::EngineState::Request;
    //     }

    //     let engine_state =
    //         self.governor
    //             .next_state(&engine_signal, &engine_command, test.engine_command_instant);

    //     log::trace!("Engine governor: {:?}", engine_state);

    //     engine_state
    // }
}

unsafe impl std::marker::Sync for VolvoD7E {}
unsafe impl std::marker::Send for VolvoD7E {}

impl super::engine::Engine for VolvoD7E {
    fn request(&self, speed: u16) -> Frame {
        self.speed_control(VolvoEngineState::Nominal, speed)
    }

    fn start(&self, speed: u16) -> Frame {
        self.speed_control(VolvoEngineState::Starting, speed)
    }

    fn stop(&self, speed: u16) -> Frame {
        self.speed_control(VolvoEngineState::Shutdown, speed)
    }
}

impl Parsable<EngineMessage> for VolvoD7E {
    fn parse(&mut self, frame: &Frame) -> Option<EngineMessage> {
        self.ems.parse(frame)
    }
}

impl super::J1939Unit for VolvoD7E {
    const VENDOR: &'static str = "volvo";
    const PRODUCT: &'static str = "d7e";

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    async fn try_accept(
        &mut self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
        runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        self.ems.try_accept(ctx, network, runtime_state).await
    }

    async fn tick(
        &self,
        _ctx: &mut super::NetDriverContext,
        _network: &crate::net::ControlNetwork,
        _runtime_state: crate::runtime::SharedOperandState,
    ) -> Result<(), super::J1939UnitError> {
        // use super::engine::Engine;

        // let request = if let Ok(request) = runtime_state.try_read() {
        //     let request = Test {
        //         engine_command: Some(request.state.engine_signal),
        //         engine_command_instant: request.state.engine_command_instant,
        //     };
        //     self.signal.set(request);
        //     request
        // } else {
        //     log::warn!("VolvoD7E tick failed to acquire runtime state lock");
        //     self.signal.get()
        // };

        // let request = self.governor_mode(request.engine_command.unwrap());
        // match request.state {
        //     crate::core::EngineState::NoRequest => {
        //         network.send(&self.request(request.rpm)).await?;
        //         ctx.tx_mark();
        //     }
        //     crate::core::EngineState::Starting => {
        //         network.send(&self.start(request.rpm)).await?;
        //         ctx.tx_mark();
        //     }
        //     crate::core::EngineState::Stopping => {
        //         network.send(&self.stop(request.rpm)).await?;
        //         ctx.tx_mark();
        //     }
        //     crate::core::EngineState::Request => {
        //         network.send(&self.request(request.rpm)).await?;
        //         ctx.tx_mark();
        //     }
        // }

        // Ok(())

        unimplemented!()
    }

    async fn trigger(
        &self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
        _runtime_state: crate::runtime::SharedOperandState,
        object: &crate::core::Object,
    ) -> Result<(), super::J1939UnitError> {
        use super::engine::Engine;

        if let crate::core::Object::Engine(engine) = object {
            // self.value.set(Test {
            //     engine_command: Some(*engine),
            //     engine_command_instant: Some(std::time::Instant::now()),
            // });

            // let request = if let Ok(request) = runtime_state.try_read() {
            //     let request = Test {
            //         engine_command: Some(request.state.engine_signal),
            //         engine_command_instant: request.state.engine_command_instant,
            //     };
            //     self.signal.set(request);
            //     request
            // } else {
            //     log::warn!("VolvoD7E tick failed to acquire runtime state lock");
            //     self.signal.get()
            // };

            // let request = self.governor_mode(request.engine_command.unwrap());
            let request = engine;
            match request.state {
                crate::core::EngineState::NoRequest => {
                    network.send(&self.request(request.rpm)).await?;
                    ctx.tx_mark();
                }
                crate::core::EngineState::Starting => {
                    network.send(&self.start(request.rpm)).await?;
                    ctx.tx_mark();
                }
                crate::core::EngineState::Stopping => {
                    network.send(&self.stop(request.rpm)).await?;
                    ctx.tx_mark();
                }
                crate::core::EngineState::Request => {
                    network.send(&self.request(request.rpm)).await?;
                    ctx.tx_mark();
                }
            }
        }

        Ok(())
    }
}
