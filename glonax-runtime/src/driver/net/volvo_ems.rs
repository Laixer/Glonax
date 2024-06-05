use std::time::Duration;

use j1939::{Frame, FrameBuilder, IdBuilder, PGN};

use crate::{
    core::{EngineState, Object, ObjectMessage},
    driver::{net::engine::Engine, EngineMessage, Governor},
    net::Parsable,
    runtime::{J1939Unit, J1939UnitError, J1939UnitOk, NetDriverContext},
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
    /// Network interface.
    #[allow(dead_code)]
    interface: String,
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
    pub fn new(interface: &str, da: u8, sa: u8) -> Self {
        Self {
            interface: interface.to_string(),
            destination_address: da,
            source_address: sa,
            ems: EngineManagementSystem::new(interface, da, sa),
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
    fn parse(&self, frame: &Frame) -> Option<EngineMessage> {
        self.ems.parse(frame)
    }
}

impl J1939Unit for VolvoD7E {
    fn vendor(&self) -> &'static str {
        "volvo"
    }

    fn product(&self) -> &'static str {
        "d7e"
    }

    fn destination(&self) -> u8 {
        self.destination_address
    }

    fn source(&self) -> u8 {
        self.source_address
    }

    fn try_recv(
        &self,
        ctx: &mut NetDriverContext,
        // network: &crate::net::ControlNetwork,
        frame: &j1939::Frame,
        signal_tx: crate::runtime::SignalSender,
    ) -> Result<J1939UnitOk, J1939UnitError> {
        self.ems.try_recv(ctx, frame, signal_tx)
    }

    fn trigger(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
        object: &Object,
    ) -> Result<(), J1939UnitError> {
        if let Object::Engine(engine_command) = object {
            ctx.set_tx_last_message(ObjectMessage::command(object.clone()));

            let engine_signal = {
                if let Some(message) = &ctx.rx_last_message() {
                    if let Object::Engine(engine) = message.object {
                        engine
                    } else {
                        crate::core::Engine::shutdown()
                    }
                } else {
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

            trace!(
                "[{}] {}: Engine: {}",
                self.interface,
                self.name(),
                governor_engine
            );

            match governor_engine.state {
                EngineState::NoRequest => {
                    tx_queue.push(self.request(governor_engine.rpm));
                }
                EngineState::Starting => {
                    tx_queue.push(self.start(governor_engine.rpm));
                }
                EngineState::Stopping => {
                    tx_queue.push(self.stop(governor_engine.rpm));
                }
                EngineState::Request => {
                    tx_queue.push(self.request(governor_engine.rpm));
                }
            }
        }

        Ok(())
    }

    fn tick(
        &self,
        ctx: &mut NetDriverContext,
        tx_queue: &mut Vec<j1939::Frame>,
    ) -> Result<(), J1939UnitError> {
        let engine_signal = {
            if let Some(message) = &ctx.rx_last_message() {
                if let Object::Engine(engine) = message.object {
                    engine
                } else {
                    crate::core::Engine::shutdown()
                }
            } else {
                crate::core::Engine::shutdown()
            }
        };

        let engine_command = {
            if let Some(message) = &ctx.tx_last_message() {
                if let Object::Engine(engine) = message.object {
                    (engine, Some(message.timestamp))
                } else {
                    (engine_signal, None)
                }
            } else {
                (engine_signal, None)
            }
        };

        let governor_engine =
            self.governor
                .next_state(&engine_signal, &engine_command.0, engine_command.1);

        trace!(
            "[{}] {}: Engine: {}",
            self.interface,
            self.name(),
            governor_engine
        );

        match governor_engine.state {
            EngineState::NoRequest => {
                tx_queue.push(self.request(governor_engine.rpm));
            }
            EngineState::Starting => {
                tx_queue.push(self.start(governor_engine.rpm));
            }
            EngineState::Stopping => {
                tx_queue.push(self.stop(governor_engine.rpm));
            }
            EngineState::Request => {
                tx_queue.push(self.request(governor_engine.rpm));
            }
        }

        Ok(())
    }
}
