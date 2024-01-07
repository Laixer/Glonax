use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

pub struct Controller {
    frame_profile: glonax::math::Linear,
    boom_profile: glonax::math::Linear,
    arm_profile: glonax::math::Linear,
    attachment_profile: glonax::math::Linear,
    frame_stop: bool,
    boom_stop: bool,
    arm_stop: bool,
    attachment_stop: bool,
    stopall: bool,
}

impl<Cnf: Configurable> Component<Cnf> for Controller {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        // TODO: Build the profile from configuration
        let frame_profile = glonax::math::Linear::new(7_000.0, 12_000.0, false);
        let boom_profile = glonax::math::Linear::new(15_000.0, 12_000.0, false);
        let arm_profile = glonax::math::Linear::new(15_000.0, 12_000.0, true);
        let attachment_profile = glonax::math::Linear::new(15_000.0, 12_000.0, false);

        Self {
            frame_profile,
            boom_profile,
            arm_profile,
            attachment_profile,
            frame_stop: false,
            boom_stop: false,
            arm_stop: false,
            attachment_stop: false,
            stopall: false,
        }
    }

    // TODO: If no errors are set, ignore the tick
    fn tick(&mut self, ctx: &mut ComponentContext, _state: &mut MachineState) {
        use glonax::core::Actuator;
        // use glonax::math::linear_motion;

        let frame_error = ctx.get(Actuator::Slew as u16);
        let boom_error = ctx.get(Actuator::Boom as u16);
        let arm_error = ctx.get(Actuator::Arm as u16);
        let attachment_error = ctx.get(Actuator::Attachment as u16);

        // let is_tri_arm_done = frame_error.is_none() && boom_error.is_none() && arm_error.is_none();
        // let is_done = is_tri_arm_done && attachment_error.is_none();

        let mut motion = vec![];

        if let Some(error) = frame_error {
            let value = self.frame_profile.update(*error);
            log::debug!("Frame error: {}, value: {}", error, value);

            motion.push((Actuator::Slew, value as i16));
            self.frame_stop = false;
        } else if !self.frame_stop {
            log::debug!("Frame error: {}, value: {}", 0, 0);

            motion.push((Actuator::Slew, 0));
            self.frame_stop = true;
        }

        if let Some(error) = boom_error {
            let value = self.boom_profile.update(*error);
            log::debug!("Boom error: {}, value: {}", error, value);

            motion.push((Actuator::Boom, value as i16));
            self.boom_stop = false;
        } else if !self.boom_stop {
            log::debug!("Boom error: {}, value: {}", 0, 0);

            motion.push((Actuator::Boom, 0));
            self.boom_stop = true;
        }

        if let Some(error) = arm_error {
            let value = self.arm_profile.update(*error);
            log::debug!("Arm error: {}, value: {}", error, value);

            motion.push((Actuator::Arm, value as i16));
            self.arm_stop = false;
        } else if !self.arm_stop {
            log::debug!("Arm error: {}, value: {}", 0, 0);

            motion.push((Actuator::Arm, 0));
            self.arm_stop = true;
        }

        if let Some(error) = attachment_error {
            let value = self.attachment_profile.update(*error);
            log::debug!("Attachment error: {}, value: {}", error, value);

            motion.push((Actuator::Attachment, value as i16));
            self.attachment_stop = false;
        } else if !self.attachment_stop {
            log::debug!("Attachment error: {}, value: {}", 0, 0);

            motion.push((Actuator::Attachment, 0));
            self.attachment_stop = true;
        }

        if !motion.is_empty() {
            ctx.commit(glonax::core::Motion::from_iter(motion));
            self.stopall = false;
        } else if !self.stopall {
            ctx.commit(glonax::core::Motion::StopAll);
            self.stopall = true;
        }
    }
}
