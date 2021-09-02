use crate::device::MetricValue;

use super::{Motion, Scancode};

pub trait Operand: Default + Clone + Send + Sync {
    fn try_from_input_device(&self, input: Scancode) -> std::result::Result<Motion, ()>;
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
    fn boot(&mut self) {}

    /// Push incoming value to program.
    ///
    /// This value can be any metric. The program
    /// must determine if and how the value is used.
    /// The id represents the device from which this
    /// value originates.
    fn push(&mut self, id: u32, value: MetricValue);

    /// Propagate the program forwards.
    ///
    /// This method returns an optional motion instruction.
    fn step(&mut self) -> Option<Motion>;

    /// Program termination condition.
    ///
    /// Check if program is finished.
    fn can_terminate(&self) -> bool;

    /// Program termination action.
    ///
    /// This is an optional method to send a last motion
    /// instruction. This method is called after `can_terminate`
    /// returns true and before the program is terminated.
    fn term_action(&self) -> Option<Motion> {
        None
    }
}
