use crate::core::{input::Scancode, motion::ToMotion, program::ProgramArgument};

use super::program::Program;

pub trait Operand: Send + Sync {
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
        program: &ProgramArgument,
    ) -> Result<Box<dyn Program<MotionPlan = Self::MotionPlan> + Send + Sync>, ()>;
}
