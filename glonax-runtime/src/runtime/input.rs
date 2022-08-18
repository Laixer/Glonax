use crate::{
    device::{Gamepad, InputDevice},
    runtime, InputConfig, Runtime,
};

use super::{operand::Operand, MotionChain};

pub struct RuntimeInput {
    input_device: Gamepad,
}

impl<'a> RuntimeInput {
    pub fn new(config: &InputConfig) -> Self {
        let input_device = Gamepad::new(std::path::Path::new(&config.device));

        Self { input_device }
    }

    pub async fn exec_service<K>(mut self, mut runtime: Runtime<K>) -> runtime::Result
    where
        K: Operand,
    {
        let mut motion_chain = MotionChain::new(&mut runtime.motion_device, &runtime.tracer);

        info!("Listen for input events");

        while let Ok(input) = self.input_device.next() {
            if let Ok(motion) = runtime.operand.try_from_input_device(input) {
                motion_chain.request(motion).await; // TOOD: Handle result
            }
        }

        Ok(())
    }
}
