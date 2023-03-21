use crate::{core::input::Scancode, Configurable};

pub trait Operand: Send + Sync {
    // type MotionPlan: ToMotion;

    /// Construct operand from configuration.
    fn from_config<C: Configurable>(config: &C) -> Self;

    // /// Try convert input scancode to motion.
    // fn try_from_input_device(&mut self, input: Scancode) -> Result<Self::MotionPlan, ()>;
}
