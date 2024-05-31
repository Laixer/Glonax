use std::time::Duration;

use j1939::{Frame, FrameBuilder, IdBuilder, PGN};

use crate::{
    core::ObjectMessage,
    driver::{EngineMessage, Governor},
    net::Parsable,
};

use super::engine::EngineManagementSystem;

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

#[derive(Clone)]
pub struct VolvoD7E {
    /// Destination address.
    destination_address: u8,
    /// Source address.
    source_address: u8,
    /// Engine management system.
    ems: EngineManagementSystem,
    /// Governor.
    governor: Governor,
}

impl VolvoD7E {
    /// Construct a new engine management system.
    pub fn new(da: u8, sa: u8) -> Self {
        Self {
            destination_address: da,
            source_address: sa,
            ems: EngineManagementSystem::new(da, sa),
            governor: Governor::new(800, 2_100, Duration::from_millis(2_000)),
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
}

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
        ipc_tx: crate::runtime::IPCSender,
    ) -> Result<(), super::J1939UnitError> {
        self.ems.try_accept(ctx, network, ipc_tx).await
    }

    async fn trigger(
        &self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
        object: &crate::core::Object,
    ) -> Result<(), super::J1939UnitError> {
        use super::engine::Engine;

        if let crate::core::Object::Engine(engine_command) = object {
            ctx.set_tx_last_message(ObjectMessage::command(object.clone()));

            let engine_signal = {
                let ctx = ctx.inner();
                if let Some(x) = &ctx.rx_last_message {
                    // log::debug!("rx_last_message: {:?}", x.object);

                    if let crate::core::Object::Engine(engine) = x.object {
                        engine
                    } else {
                        crate::core::Engine::shutdown()
                    }
                } else {
                    // log::debug!("rx_last_message: None");
                    crate::core::Engine::shutdown()
                }
            };

            let engine_command = {
                if engine_command.rpm > 0 {
                    crate::core::Engine::from_rpm(engine_command.rpm)
                } else {
                    crate::core::Engine::shutdown()
                }
            };

            let governor_engine = self
                .governor
                .next_state(&engine_signal, &engine_command, None);

            trace!("Engine: {}", governor_engine);

            match governor_engine.state {
                crate::core::EngineState::NoRequest => {
                    network.send(&self.request(governor_engine.rpm)).await?;
                }
                crate::core::EngineState::Starting => {
                    network.send(&self.start(governor_engine.rpm)).await?;
                }
                crate::core::EngineState::Stopping => {
                    network.send(&self.stop(governor_engine.rpm)).await?;
                }
                crate::core::EngineState::Request => {
                    network.send(&self.request(governor_engine.rpm)).await?;
                }
            }
        }

        Ok(())
    }

    async fn tick(
        &self,
        ctx: &mut super::NetDriverContext,
        network: &crate::net::ControlNetwork,
    ) -> Result<(), super::J1939UnitError> {
        use super::engine::Engine;

        let engine_signal = {
            let ctx = ctx.inner();
            if let Some(x) = &ctx.rx_last_message {
                // log::debug!("rx_last_message: {:?}", x.object);

                if let crate::core::Object::Engine(engine) = x.object {
                    engine
                } else {
                    crate::core::Engine::shutdown()
                }
            } else {
                // log::debug!("rx_last_message: None");
                crate::core::Engine::shutdown()
            }
        };

        let engine_command = {
            let ctx = ctx.inner();
            if let Some(x) = &ctx.tx_last_message {
                // log::debug!("tx_last_message: {:?}", x.object);

                if let crate::core::Object::Engine(engine) = x.object {
                    (engine, Some(x.timestamp))
                } else {
                    (engine_signal, None)
                }
            } else {
                // log::debug!("tx_last_message: None");
                (engine_signal, None)
            }
        };

        let governor_engine =
            self.governor
                .next_state(&engine_signal, &engine_command.0, engine_command.1);

        trace!("Engine: {}", governor_engine);

        match governor_engine.state {
            crate::core::EngineState::NoRequest => {
                network.send(&self.request(governor_engine.rpm)).await?;
            }
            crate::core::EngineState::Starting => {
                network.send(&self.start(governor_engine.rpm)).await?;
            }
            crate::core::EngineState::Stopping => {
                network.send(&self.stop(governor_engine.rpm)).await?;
            }
            crate::core::EngineState::Request => {
                network.send(&self.request(governor_engine.rpm)).await?;
            }
        }

        Ok(())
    }
}
