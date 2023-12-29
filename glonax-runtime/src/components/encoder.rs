use crate::{
    core::Actuator,
    net::{Encoder, EncoderMessage},
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

pub struct EncoderSimulator {
    control_devices: [(u8, Actuator, Encoder); 4],
}

impl<Cnf: Configurable> Component<Cnf> for EncoderSimulator {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        log::debug!("Starting encoder service");

        let encoder_frame = Encoder::new(2_500, (0, 6_280), true, false);
        let encoder_boom = Encoder::new(5_000, (0, 1_832), false, false);
        let encoder_arm = Encoder::new(5_000, (685, 2_760), false, true);
        let encoder_attachment = Encoder::new(5_000, (0, 3_100), false, false);

        // TODO: Make this into a struct and move it to /robot
        let control_devices = [
            (0x6A, Actuator::Slew, encoder_frame),
            (0x6B, Actuator::Boom, encoder_boom),
            (0x6C, Actuator::Arm, encoder_arm),
            (0x6D, Actuator::Attachment, encoder_attachment),
        ];

        Self { control_devices }
    }

    fn tick(&mut self, _ctx: &mut ComponentContext, state: &mut MachineState) {
        for (id, actuator, encoder) in self.control_devices.iter_mut() {
            let velocity = state.ecu_state.speed(actuator);
            let position = state.ecu_state.position(actuator);

            let position = encoder.position(position, velocity);

            EncoderMessage::from_position(*id, position).fill2(&mut state.pose);

            state.ecu_state.set_position(actuator, position);
        }
    }
}
