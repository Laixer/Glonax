use std::time::Instant;

use crate::{
    core::{input::Scancode, motion::ToMotion},
    signal::SignalReader,
};

pub trait Operand: Clone + Send + Sync {
    type MotionPlan: ToMotion;

    /// Construct operand from configuration.
    fn from_config<C: crate::config::Configurable>(config: &C) -> Self;

    /// Try convert input scancode to motion.
    fn try_from_input_device(&mut self, input: Scancode) -> Result<Self::MotionPlan, ()>;
}

pub trait ProgramFactory {
    type MotionPlan: ToMotion;

    // TODO: Handle result.
    /// Fetch program by identifier.
    ///
    /// The factory method returns a pointer to the program which
    /// will be execured by the runtime. The program identifier
    /// is a per kernel unique program identifier.
    fn fetch_program(
        &self,
        id: i32,
        params: Parameter,
    ) -> Result<Box<dyn Program<MotionPlan = Self::MotionPlan> + Send + Sync>, ()>;
}

pub struct Context {
    /// Time of start of the program.
    pub start: Instant,
    /// Time of last step.
    pub last_step: Instant,
    /// Total step count.
    pub step_count: usize,
    /// Signal reader.
    pub reader: SignalReader,
}

impl Context {
    /// Construct new program context.
    pub fn new(reader: SignalReader) -> Self {
        Self {
            start: Instant::now(),
            last_step: Instant::now(),
            step_count: 0,
            reader,
        }
    }
}

pub type Parameter = Vec<f32>;

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
