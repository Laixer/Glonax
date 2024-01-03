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

    // TODO: Write the result back to the component context
    // TODO: If no errors are set, ignore the tick
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        use glonax::core::Actuator;
        use glonax::math::linear_motion;

        let frame_error = *ctx.get(Actuator::Slew as u16).unwrap_or(&0.0);
        let boom_error = *ctx.get(Actuator::Boom as u16).unwrap_or(&0.0);
        let arm_error = *ctx.get(Actuator::Arm as u16).unwrap_or(&0.0);
        let attachment_error = *ctx.get(Actuator::Attachment as u16).unwrap_or(&0.0);

        log::debug!(
            "Frame error: {}, boom error: {}, arm error: {}, attachment error: {}",
            frame_error,
            boom_error,
            arm_error,
            attachment_error
        );

        let frame_profile = glonax::math::Linear::new(7_000.0, 12_000.0, false);
        let boom_profile = glonax::math::Linear::new(15_000.0, 12_000.0, false);
        let arm_profile = glonax::math::Linear::new(15_000.0, 12_000.0, true);
        let attachment_profile = glonax::math::Linear::new(15_000.0, 12_000.0, false);

        // let frame_value = linear_motion(frame_error, 0.01, 5_000.0, 12_000.0, false);
        // let boom_value = linear_motion(boom_error, 0.01, 5_000.0, 12_000.0, false);
        // let arm_value = linear_motion(arm_error, 0.01, 5_000.0, 12_000.0, false);
        // let attachment_value = linear_motion(attachment_error, 0.01, 5_000.0, 12_000.0, false);

        // let is_tri_arm_done = frame_value.is_none() && boom_value.is_none() && arm_value.is_none();
        // let is_done = is_tri_arm_done && attachment_value.is_none();

        // if is_done {
        //     ctx.commit(glonax::core::Motion::StopAll);

        //     // TODO: Keep the target, but set 'finished' to true
        //     state.target = None;
        // } else {
        //     ctx.commit(glonax::core::Motion::from_iter(vec![
        //         (Actuator::Slew, frame_value.unwrap_or(0)),
        //         (Actuator::Boom, boom_value.unwrap_or(0)),
        //         (Actuator::Arm, arm_value.unwrap_or(0)),
        //         (Actuator::Attachment, attachment_value.unwrap_or(0)),
        //     ]));
        // }
    }
}
