use glonax::{
    core::{Actuator, Motion, Object},
    runtime::{CommandSender, Component, ComponentContext, IPCReceiver},
};

struct ActuatorMotionEvent {
    actuator: Actuator,
    error: f32,
    value: i16,
}

struct ActuatorState {
    profile: glonax::math::Linear,
    actuator: Actuator,
    stop: bool,
}

impl ActuatorState {
    fn from_profile(actuator: Actuator, profile: glonax::math::Linear) -> Self {
        Self {
            profile,
            actuator,
            stop: false,
        }
    }

    fn update(&mut self, error: Option<f32>) -> Option<ActuatorMotionEvent> {
        if let Some(error) = error {
            self.stop = false;

            Some(ActuatorMotionEvent {
                actuator: self.actuator,
                error,
                value: self.profile.update(error) as i16,
            })
        } else if !self.stop {
            self.stop = true;

            Some(ActuatorMotionEvent {
                actuator: self.actuator,
                error: 0.0,
                value: Motion::POWER_NEUTRAL,
            })
        } else {
            None
        }
    }
}

pub struct Controller {
    frame_state: ActuatorState,
    boom_state: ActuatorState,
    arm_state: ActuatorState,
    attachment_state: ActuatorState,
    stopall: bool,
}

impl<Cnf: Clone> Component<Cnf> for Controller {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        // TODO: Build the profile from configuration
        let frame_profile = glonax::math::Linear::new(7_000.0, 12_000.0, false);
        let boom_profile = glonax::math::Linear::new(15_000.0, 12_000.0, false);
        let arm_profile = glonax::math::Linear::new(15_000.0, 12_000.0, true);
        let attachment_profile = glonax::math::Linear::new(15_000.0, 12_000.0, false);

        let frame_state = ActuatorState::from_profile(Actuator::Slew, frame_profile);
        let boom_state = ActuatorState::from_profile(Actuator::Boom, boom_profile);
        let arm_state = ActuatorState::from_profile(Actuator::Arm, arm_profile);
        let attachment_state =
            ActuatorState::from_profile(Actuator::Attachment, attachment_profile);

        Self {
            frame_state,
            boom_state,
            arm_state,
            attachment_state,
            stopall: false,
        }
    }

    fn tick(
        &mut self,
        ctx: &mut ComponentContext,
        _ipc_rx: std::rc::Rc<IPCReceiver>,
        command_tx: CommandSender,
    ) {
        let frame_error = ctx.actuators.get(&(Actuator::Slew as u16));
        let boom_error = ctx.actuators.get(&(Actuator::Boom as u16));
        let arm_error = ctx.actuators.get(&(Actuator::Arm as u16));
        let attachment_error = ctx.actuators.get(&(Actuator::Attachment as u16));

        // let is_tri_arm_done = frame_error.is_none() && boom_error.is_none() && arm_error.is_none();
        // let is_done = is_tri_arm_done && attachment_error.is_none();

        let mut motion = vec![];

        if let Some(event) = self.frame_state.update(frame_error.copied()) {
            log::debug!(
                "{:?} error: {}, value: {}",
                event.actuator,
                event.error,
                event.value
            );

            motion.push((event.actuator, event.value));
        }

        if let Some(event) = self.boom_state.update(boom_error.copied()) {
            log::debug!(
                "{:?} error: {}, value: {}",
                event.actuator,
                event.error,
                event.value
            );

            motion.push((event.actuator, event.value));
        }

        if let Some(event) = self.arm_state.update(arm_error.copied()) {
            log::debug!(
                "{:?} error: {}, value: {}",
                event.actuator,
                event.error,
                event.value
            );

            motion.push((event.actuator, event.value));
        }

        if let Some(event) = self.attachment_state.update(attachment_error.copied()) {
            log::debug!(
                "{:?} error: {}, value: {}",
                event.actuator,
                event.error,
                event.value
            );

            motion.push((event.actuator, event.value));
        }

        // if let Some(error) = frame_error {
        //     let value = self.frame_profile.update(*error);
        //     log::debug!("Frame error: {}, value: {}", error, value);

        //     motion.push((Actuator::Slew, value as i16));
        //     self.frame_stop = false;
        // } else if !self.frame_stop {
        //     log::debug!("Frame error: {}, value: {}", 0, Motion::POWER_NEUTRAL);

        //     motion.push((Actuator::Slew, Motion::POWER_NEUTRAL));
        //     self.frame_stop = true;
        // }

        // if let Some(error) = boom_error {
        //     let value = self.boom_profile.update(*error);
        //     log::debug!("Boom error: {}, value: {}", error, value);

        //     motion.push((Actuator::Boom, value as i16));
        //     self.boom_stop = false;
        // } else if !self.boom_stop {
        //     log::debug!("Boom error: {}, value: {}", 0, Motion::POWER_NEUTRAL);

        //     motion.push((Actuator::Boom, Motion::POWER_NEUTRAL));
        //     self.boom_stop = true;
        // }

        // if let Some(error) = arm_error {
        //     let value = self.arm_profile.update(*error);
        //     log::debug!("Arm error: {}, value: {}", error, value);

        //     motion.push((Actuator::Arm, value as i16));
        //     self.arm_stop = false;
        // } else if !self.arm_stop {
        //     log::debug!("Arm error: {}, value: {}", 0, Motion::POWER_NEUTRAL);

        //     motion.push((Actuator::Arm, Motion::POWER_NEUTRAL));
        //     self.arm_stop = true;
        // }

        // if let Some(error) = attachment_error {
        //     let value = self.attachment_profile.update(*error);
        //     log::debug!("Attachment error: {}, value: {}", error, value);

        //     motion.push((Actuator::Attachment, value as i16));
        //     self.attachment_stop = false;
        // } else if !self.attachment_stop {
        //     log::debug!("Attachment error: {}, value: {}", 0, Motion::POWER_NEUTRAL);

        //     motion.push((Actuator::Attachment, Motion::POWER_NEUTRAL));
        //     self.attachment_stop = true;
        // }

        if !motion.is_empty() {
            let motion_command = Motion::from_iter(motion);
            if let Err(e) = command_tx.try_send(Object::Motion(motion_command.clone())) {
                log::error!("Failed to send motion command: {}", e);
            }

            ctx.machine.motion_command = Some(motion_command);
            ctx.machine.motion_command_instant = Some(std::time::Instant::now());

            self.stopall = false;
        } else if !self.stopall {
            let motion_command = Motion::StopAll;
            if let Err(e) = command_tx.try_send(Object::Motion(motion_command.clone())) {
                log::error!("Failed to send motion command: {}", e);
            }

            ctx.machine.motion_command = Some(motion_command);
            ctx.machine.motion_command_instant = Some(std::time::Instant::now());

            self.stopall = true;
        }
    }
}
