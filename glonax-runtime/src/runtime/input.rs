use crate::{
    device::{Gamepad, Hcu, InputDevice},
    runtime, InputConfig, RuntimeContext,
};

use super::{operand::Operand, MotionChain};

pub struct RuntimeInput {
    input_device: Gamepad,
}

impl RuntimeInput {
    pub fn new(config: &InputConfig) -> Self {
        Self {
            input_device: Gamepad::new(std::path::Path::new(&config.device)),
        }
    }

    pub async fn exec_service<K: Operand>(
        mut self,
        mut runtime: RuntimeContext<K>,
    ) -> runtime::Result {
        let mut motion_device = runtime.core_device.new_gateway_device::<Hcu>();

        let mut motion_chain = MotionChain::new(&mut motion_device, &runtime.tracer);

        info!("Listen for input events");

        while let Ok(input) = self.input_device.next() {
            if let Ok(motion) = runtime.operand.try_from_input_device(input) {
                // use crate::core::motion::ToMotion;

                // let mo = motion.to_motion();

                // let gcx = unsafe {
                //     std::slice::from_raw_parts(
                //         (&mo as *const crate::core::motion::Motion) as *const u8,
                //         std::mem::size_of::<crate::core::motion::Motion>(),
                //     )
                // };

                // sock.send_to(gcx, "0.0.0.0:54910").await.unwrap();

                motion_chain.request(motion).await; // TOOD: Handle result
            }
        }

        Ok(())
    }
}
