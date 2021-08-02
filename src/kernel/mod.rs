pub mod machine;

use crate::device::MetricValue;

pub trait Program {
    type Motion;

    /// Push incoming value into program.
    ///
    /// This value can be any metric. The program
    /// must determine if and how the value is used.
    /// The id represents the device from which this
    /// value originates.
    fn push(&mut self, id: u32, value: MetricValue);

    /// Propagate the program forwards.
    ///
    /// This method returns an optional motion instruction.
    fn step(&mut self) -> Option<Self::Motion>;

    /// Program termination condition.
    ///
    /// Check if program is finished.
    fn can_terminate(&self) -> bool;

    /// Program termination action.
    ///
    /// This is an optional method to send a last motion
    /// instruction.
    fn term_action(&self) -> Option<Self::Motion> {
        None
    }
}
