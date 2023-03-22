use crate::Configurable;

pub trait Operand: Send + Sync {
    /// Construct operand from configuration.
    fn from_config<C: Configurable>(config: &C) -> Self;
}
