use std::sync::atomic::Ordering;

use glonax::{
    net::EncoderMessage,
    runtime::{Component, ComponentContext},
    RobotState,
};

use crate::state::Excavator;

pub struct EncoderSimulator {
    control_devices: [(u8, glonax::core::Actuator, glonax::net::Encoder); 4],
}

// impl<R: RobotState> Component<R> for EncoderSimService {
impl Component<Excavator> for EncoderSimulator {
    fn tick(&mut self, _ctx: &mut ComponentContext, state: &mut Excavator) {
        for (id, actuator, encoder) in self.control_devices.iter_mut() {
            // 1st derivative of position
            let velocity = state.ecu_state.speed[*actuator as usize].load(Ordering::SeqCst);
            let position = state.ecu_state.position[*actuator as usize].load(Ordering::SeqCst);

            let position = encoder.position(position, velocity);

            EncoderMessage::from_position(*id, position).fill2(state.pose_mut());

            state.ecu_state.position[*actuator as usize]
                .store(position, std::sync::atomic::Ordering::Relaxed);

            // log::debug!("0x{:X?} Encoder position: {}", id, position);
        }
    }
}

impl Default for EncoderSimulator {
    fn default() -> Self {
        log::debug!("Starting encoder service");

        let encoder_frame = glonax::net::Encoder::new(2_500, (0, 6_280), true, false);
        let encoder_boom = glonax::net::Encoder::new(5_000, (0, 1_832), false, false);
        let encoder_arm = glonax::net::Encoder::new(5_000, (685, 2_760), false, true);
        let encoder_attachment = glonax::net::Encoder::new(5_000, (0, 3_100), false, false);

        let control_devices = [
            (0x6A, glonax::core::Actuator::Slew, encoder_frame),
            (0x6B, glonax::core::Actuator::Boom, encoder_boom),
            (0x6C, glonax::core::Actuator::Arm, encoder_arm),
            (0x6D, glonax::core::Actuator::Attachment, encoder_attachment),
        ];

        Self { control_devices }
    }
}
