pub mod input;
pub mod metric;
pub mod motion;
pub mod operand;
pub mod position;

pub trait Identity {
    /// Introduction message.
    ///
    /// Returns a string to introduce the object for the first time and
    /// should only be called once.
    fn intro() -> String;
}
