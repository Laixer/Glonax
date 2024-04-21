use crate::{
    driver::VirtualEncoder,
    runtime::{Service, ServiceContext, SharedOperandState},
};

pub struct EncoderSimulator {
    encoder_list: [(u8, crate::core::Actuator, VirtualEncoder); 4],
}

impl<C> Service<C> for EncoderSimulator {
    fn new(_config: C) -> Self
    where
        Self: Sized,
    {
        let encoder_frame = VirtualEncoder::new(2_500, (0, 6_280), true, false);
        let encoder_boom = VirtualEncoder::new(5_000, (0, 1_832), false, false);
        let encoder_arm = VirtualEncoder::new(5_000, (685, 2_760), false, true);
        let encoder_attachment = VirtualEncoder::new(5_000, (0, 3_100), false, false);

        let encoder_list = [
            (0x6A, crate::core::Actuator::Slew, encoder_frame),
            (0x6B, crate::core::Actuator::Boom, encoder_boom),
            (0x6C, crate::core::Actuator::Arm, encoder_arm),
            (0x6D, crate::core::Actuator::Attachment, encoder_attachment),
        ];

        Self { encoder_list }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("encoder simulator")
    }

    async fn tick(&mut self, runtime_state: SharedOperandState) {
        // let frame = &mut self.encoder_list[0];
        // let position = frame.1.position_from_angle(100_f32.to_radians());

        // state.encoders.insert(0x6A, 0.0);
        // state.encoders.insert(0x6B, 0.0);
        // state.encoders.insert(0x6C, 0.0);
        // state.encoders.insert(0x6D, 0.0);

        let mut runtime_state = runtime_state.write().await;
        for (id, actuator, encoder) in self.encoder_list.iter_mut() {
            let velocity = runtime_state.state.ecu_state.speed(actuator);
            let position = runtime_state.state.ecu_state.position(actuator);

            let position = encoder.position(position, velocity);

            runtime_state.state.encoders.insert(*id, position as f32);

            runtime_state
                .state
                .ecu_state
                .set_position(actuator, position);
        }
    }
}
