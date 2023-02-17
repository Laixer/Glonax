use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    core::{input::Scancode, motion::ToMotion},
    Configurable,
};

use super::program::Program;

pub trait Operand: Send + Sync {
    type MotionPlan: ToMotion;

    /// Construct operand from configuration.
    fn from_config<C: Configurable>(config: &C) -> Self;

    /// Try convert input scancode to motion.
    fn try_from_input_device(&mut self, input: Scancode) -> Result<Self::MotionPlan, ()>;
}

/// The function trait defines a kernel function descriptor. The function descripter is used
/// outside the kernel by the runtime and other systems of the framework when referring to
/// a operand function.
pub trait FunctionTrait: Send + Display + Sync + Serialize + for<'a> Deserialize<'a> {
    fn name(&self) -> String;
}

pub trait FunctionFactory {
    type MotionPlan: ToMotion;
    type FunctionType: FunctionTrait;

    fn parse_function(&self, ident: &str, parameters: Vec<f32>) -> Self::FunctionType;

    // TODO: Handle result.
    /// Fetch function from the operand.
    ///
    /// The factory method returns a reference to the function which
    /// can be execured by the runtime.
    fn fetch_function(
        &self,
        argument: &Self::FunctionType,
    ) -> Result<Box<dyn Program<MotionPlan = Self::MotionPlan> + Send + Sync>, ()>;
}
