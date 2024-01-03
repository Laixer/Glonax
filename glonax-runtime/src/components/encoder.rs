use crate::{
    device::VirtualEncoder,
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

pub struct EncoderSimulator {
    encoder_list: [(u8, VirtualEncoder); 4],
}

impl<Cnf: Configurable> Component<Cnf> for EncoderSimulator {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        log::debug!("Starting encoder component");

        let encoder_frame = VirtualEncoder::new(2_500, (0, 6_280), true, false);
        let encoder_boom = VirtualEncoder::new(5_000, (0, 1_832), false, false);
        let encoder_arm = VirtualEncoder::new(5_000, (685, 2_760), false, true);
        let encoder_attachment = VirtualEncoder::new(5_000, (0, 3_100), false, false);

        let encoder_list = [
            (0x6A, encoder_frame),
            (0x6B, encoder_boom),
            (0x6C, encoder_arm),
            (0x6D, encoder_attachment),
        ];

        Self { encoder_list }
    }

    fn tick(&mut self, _ctx: &mut ComponentContext, state: &mut MachineState) {
        // for (id, actuator, encoder) in self.control_devices.iter_mut() {
        //     let velocity = state.ecu_state.speed(actuator);
        //     let position = state.ecu_state.position(actuator);

        // let position = encoder.position(position, velocity);

        // state.ecu_state.set_position(actuator, position);
        // }

        let frame = &mut self.encoder_list[0];
        let position = frame.1.position_from_angle(100_f32.to_radians());

        state.encoders.insert(frame.0, position as f32);
        state.pose.set_node_position(frame.0, position);
    }
}
