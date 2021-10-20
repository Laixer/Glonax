use std::time::Instant;

use crate::{input::Scancode, metric::MetricValue, motion::Motion};

pub trait Operand: Default + Clone + Send + Sync {
    /// Try convert input scancode to motion.
    fn try_from_input_device(&self, input: Scancode) -> Result<Motion, ()>;

    /// Fetch program by identifier.
    ///
    /// The factory method returns a pointer to the program which
    /// will be execured by the runtime. The program identifier
    /// is a per kernel unique program identifier.
    fn fetch_program(&self, id: i32) -> Box<dyn Program + Send + Sync>;
}

pub struct Context {
    /// Time of start of the program.
    pub start: Instant,
    /// Time of last step.
    pub last_step: Instant,
}

impl Context {
    /// Construct new program context.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            last_step: Instant::now(),
        }
    }
}

/// Program trait.
///
/// A program is run on the runtime. It reads input from various
/// sources and returns an optional motion instruction. A program
/// is run to completion. The completion condition is polled on
/// every cycle.
pub trait Program {
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
    fn push(&mut self, _id: u32, _: MetricValue, _context: &mut Context) {}

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
