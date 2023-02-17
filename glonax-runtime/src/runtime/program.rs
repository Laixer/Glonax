use std::time::Instant;

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
