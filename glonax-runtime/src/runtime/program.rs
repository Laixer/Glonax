use std::{sync::Arc, time::Instant};

use tokio::sync::mpsc;

use crate::{core::motion::ToMotion, signal::SignalManager};

pub struct Context<'a> {
    /// Time of start of the program.
    pub start: Instant,
    /// Time of last step.
    pub last_step: Instant,
    /// Signal reader.
    pub reader: &'a mut SignalManager,
}

impl<'a> Context<'a> {
    /// Construct new program context.
    pub fn new(reader: &'a mut SignalManager) -> Self {
        Self {
            start: Instant::now(),
            last_step: Instant::now(),
            reader,
        }
    }
}

/// Program trait.
///
/// A program is run on the runtime. It reads input from various
/// sources and returns an optional motion instruction. A program
/// is run to completion. The completion condition is polled on
/// every cycle.
#[async_trait::async_trait]
pub trait Program {
    type MotionPlan: ToMotion;

    /// Boot the program.
    ///
    /// This method is called when the runtime accepted
    /// this progam and started its routine.
    fn boot(&mut self, _context: &mut Context) -> Option<Self::MotionPlan> {
        None
    }

    /// Propagate the program forwards.
    ///
    /// This method returns an optional motion instruction.
    async fn step(&mut self, context: &mut Context) -> Option<Self::MotionPlan>;

    /// Program termination condition.
    ///
    /// Check if program is finished.
    fn can_terminate(&self, context: &mut Context) -> bool;

    /// Program termination action.
    ///
    /// This is an optional method to send a last motion
    /// instruction. This method is called after `can_terminate`
    /// returns true and before the program is terminated.
    fn term_action(&self, _context: &mut Context) -> Option<Self::MotionPlan> {
        None
    }
}

const TOPIC: &str = "command/program";

pub struct ProgramManager<T: crate::runtime::operand::FunctionTrait> {
    client: Arc<rumqttc::AsyncClient>,
    queue: (mpsc::Sender<T>, mpsc::Receiver<T>),
}

impl<T: crate::runtime::operand::FunctionTrait> ProgramManager<T> {
    pub(super) fn new(client: Arc<rumqttc::AsyncClient>) -> Self {
        Self {
            client,
            queue: mpsc::channel(1024),
        }
    }

    pub(super) fn adapter(&self) -> ProgramQueueAdapter<T> {
        ProgramQueueAdapter::<T> {
            queue: self.queue.0.clone(),
        }
    }

    pub async fn publish(&mut self, program: T) {
        if let Ok(str_payload) = serde_json::to_string(&program) {
            match self
                .client
                .publish(
                    TOPIC,
                    rumqttc::QoS::ExactlyOnce,
                    false,
                    str_payload.as_bytes(),
                )
                .await
            {
                // Ok(_) => trace!("Published program: {:?}", program),
                Ok(_) => trace!("Published program"),
                Err(_) => warn!("Failed to publish program"),
            }
        }
    }

    pub(super) async fn recv(&mut self) -> Option<T> {
        self.queue.1.recv().await
    }
}

pub(super) struct ProgramQueueAdapter<T: crate::runtime::operand::FunctionTrait> {
    queue: mpsc::Sender<T>,
}

#[async_trait::async_trait]
impl<T: crate::runtime::operand::FunctionTrait> super::QueueAdapter for ProgramQueueAdapter<T> {
    fn topic(&self) -> &str {
        self::TOPIC
    }

    fn qos(&self) -> rumqttc::QoS {
        rumqttc::QoS::ExactlyOnce
    }

    async fn parse(&mut self, event: &rumqttc::Publish) {
        if let Ok(str_payload) = std::str::from_utf8(&event.payload) {
            if let Ok(program) = serde_json::from_str::<T>(str_payload) {
                if self.queue.try_send(program).is_err() {
                    warn!("Program queue reached maximum capacity");
                }
            }
        }
    }
}
