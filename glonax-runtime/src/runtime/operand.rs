use std::time::Instant;

use glonax_core::{input::Scancode, motion::Motion};

use super::{
    pipeline::{Signal, Sink},
    RuntimeSession,
};

pub trait Operand: Default + Clone + Send + Sync {
    /// Try convert input scancode to motion.
    fn try_from_input_device(&mut self, input: Scancode) -> Result<Motion, ()>;

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
    ) -> Result<Box<dyn Program + Send + Sync>, ()>;
}

pub struct Context {
    /// Time of start of the program.
    pub start: Instant,
    /// Time of last step.
    pub last_step: Instant,
    /// Total step count.
    pub step_count: usize,
    /// Runtime session.
    pub session: RuntimeSession,
}

impl Context {
    /// Construct new program context.
    pub fn new(session: RuntimeSession) -> Self {
        Self {
            start: Instant::now(),
            last_step: Instant::now(),
            step_count: 0,
            session,
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
pub trait Program: Sink {
    /// Boot the program.
    ///
    /// This method is called when the runtime accepted
    /// this progam and started its routine.
    fn boot(&mut self, _context: &mut Context) {}

    /// Push incoming value to program.
    ///
    /// This value can be any metric. The program
    /// must determine if and how the value is used.
    /// The id represents the device from which this
    /// value originates.
    fn push(&mut self, _domain: Signal) {}

    /// Propagate the program forwards.
    ///
    /// This method returns an optional motion instruction.
    fn step(&mut self, context: &mut Context) -> Option<Motion>;

    /// Program termination condition.
    ///
    /// Check if program is finished.
    fn can_terminate(&self, context: &mut Context) -> bool;

    /// Program termination action.
    ///
    /// This is an optional method to send a last motion
    /// instruction. This method is called after `can_terminate`
    /// returns true and before the program is terminated.
    fn term_action(&self, _context: &mut Context) -> Option<Motion> {
        None
    }
}

/// Blanket implementation of the oipeline sink for all programs.
///
/// The `Program` trait offers a default implementation for the domain push
/// so that it remains optional to adopt the domain value.
impl<T: Program> Sink for T {
    #[inline]
    fn distribute(&mut self, domain: Signal) {
        self.push(domain)
    }
}
