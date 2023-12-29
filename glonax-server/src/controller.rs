use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

pub struct Controller;

impl<Cnf: Configurable> Component<Cnf> for Controller {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        let frame_error = *ctx.get(glonax::core::Actuator::Slew as u16).unwrap_or(&0.0);
        let boom_error = *ctx.get(glonax::core::Actuator::Boom as u16).unwrap_or(&0.0);
        let arm_error = *ctx.get(glonax::core::Actuator::Arm as u16).unwrap_or(&0.0);
        let attachment_error = *ctx
            .get(glonax::core::Actuator::Attachment as u16)
            .unwrap_or(&0.0);

        let frame_value = glonax::math::linear_motion(frame_error, 0.01, 5_000.0, 12_000.0, false);
        let boom_value = glonax::math::linear_motion(boom_error, 0.01, 5_000.0, 12_000.0, false);
        let arm_value = glonax::math::linear_motion(arm_error, 0.01, 5_000.0, 12_000.0, false);
        let attachment_value =
            glonax::math::linear_motion(attachment_error, 0.01, 5_000.0, 12_000.0, false);

        let is_tri_arm_done = frame_value.is_none() && boom_value.is_none() && arm_value.is_none();
        let is_done = is_tri_arm_done && attachment_value.is_none();

        if is_done {
            ctx.commit(glonax::core::Motion::StopAll);

            // TODO: Keep the target, but set 'finished' to true
            state.target = None;
        } else {
            ctx.commit(glonax::core::Motion::from_iter(vec![
                (glonax::core::Actuator::Slew, frame_value.unwrap_or(0)),
                (glonax::core::Actuator::Boom, boom_value.unwrap_or(0)),
                (glonax::core::Actuator::Arm, arm_value.unwrap_or(0)),
                (
                    glonax::core::Actuator::Attachment,
                    attachment_value.unwrap_or(0),
                ),
            ]));
        }
    }
}
