use nalgebra::Vector3;

use crate::{
    core::{Actuator, Control, Engine, MachineType, Motion, Object, Target},
    math::Linear,
    runtime::{CommandSender, NullConfig, Service, ServiceContext, SignalReceiver},
    world::{Actor, ActorBuilder, ActorSegment},
};

const ROBOT_ACTOR_NAME: &str = "volvo_ec240cl";

const ENCODER_FRAME: u8 = 0x6A;
const ENCODER_BOOM: u8 = 0x6B;
const ENCODER_ARM: u8 = 0x6C;
const ENCODER_ATTACHMENT: u8 = 0x6D;

mod experimental {
    use crate::{
        core::{Actuator, Motion},
        math::Linear,
    };

    pub(super) struct ActuatorMotionEvent {
        pub actuator: Actuator,
        pub error: f32,
        pub value: i16,
    }

    pub(super) struct ActuatorState {
        profile: Linear,
        actuator: Actuator,
        stop: bool,
    }

    impl ActuatorState {
        pub(super) fn bind(actuator: Actuator, profile: Linear) -> Self {
            Self {
                profile,
                actuator,
                stop: false,
            }
        }

        pub(super) fn update(&mut self, error: Option<f32>) -> Option<ActuatorMotionEvent> {
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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DirectorOperation {
    Disabled,
    Supervised,
    Autonomous,
}

impl std::fmt::Display for DirectorOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectorOperation::Disabled => write!(f, "disabled"),
            DirectorOperation::Supervised => write!(f, "supervised"),
            DirectorOperation::Autonomous => write!(f, "autonomous"),
        }
    }
}

pub struct Director {
    actor: Actor,
    operation: DirectorOperation,
    target: Vec<Target>,
    frame_state: experimental::ActuatorState,
    boom_state: experimental::ActuatorState,
    arm_state: experimental::ActuatorState,
    attachment_state: experimental::ActuatorState,
}

impl Director {
    /// Sends a series of control commands to perform an emergency stop.
    ///
    /// # Arguments
    ///
    /// * `command_tx` - The command sender used to send control commands.
    fn command_emergency_stop(command_tx: &CommandSender) {
        let control_command = Control::HydraulicLock(true);
        if let Err(e) = command_tx.send(Object::Control(control_command)) {
            log::error!("Failed to send control command: {}", e);
        }

        let motion_command = Motion::StopAll;
        if let Err(e) = command_tx.send(Object::Motion(motion_command)) {
            log::error!("Failed to send motion command: {}", e);
        }

        let control_command = Control::HydraulicBoost(false);
        if let Err(e) = command_tx.send(Object::Control(control_command)) {
            log::error!("Failed to send control command: {}", e);
        }

        let control_command = Control::MachineTravelAlarm(true);
        if let Err(e) = command_tx.send(Object::Control(control_command)) {
            log::error!("Failed to send control command: {}", e);
        }

        let control_command = Control::MachineStrobeLight(true);
        if let Err(e) = command_tx.send(Object::Control(control_command)) {
            log::error!("Failed to send control command: {}", e);
        }

        let engine_command = Engine::shutdown();
        if let Err(e) = command_tx.send(Object::Engine(engine_command)) {
            log::error!("Failed to send engine command: {}", e);
        }
    }
}

impl Service<NullConfig> for Director {
    fn new(_: NullConfig) -> Self
    where
        Self: Sized,
    {
        // TODO: Build the actor from configuration and machine instance
        let actor = ActorBuilder::new(ROBOT_ACTOR_NAME, MachineType::Excavator)
            .attach_segment(
                "undercarriage",
                ActorSegment::new(Vector3::new(0.0, 0.0, 0.0)),
            )
            .attach_segment("frame", ActorSegment::new(Vector3::new(-4.0, 5.0, 107.0)))
            .attach_segment("boom", ActorSegment::new(Vector3::new(4.0, 20.0, 33.0)))
            .attach_segment("arm", ActorSegment::new(Vector3::new(510.0, 20.0, 5.0)))
            .attach_segment(
                "attachment",
                ActorSegment::new(Vector3::new(310.0, -35.0, 45.0)),
            )
            .build();

        // TODO: Build the profile from configuration
        let frame_profile = Linear::new(7_000.0, 12_000.0, false);
        let boom_profile = Linear::new(15_000.0, 12_000.0, false);
        let arm_profile = Linear::new(15_000.0, 12_000.0, true);
        let attachment_profile = Linear::new(15_000.0, 12_000.0, false);

        let frame_state = experimental::ActuatorState::bind(Actuator::Slew, frame_profile);
        let boom_state = experimental::ActuatorState::bind(Actuator::Boom, boom_profile);
        let arm_state = experimental::ActuatorState::bind(Actuator::Arm, arm_profile);
        let attachment_state =
            experimental::ActuatorState::bind(Actuator::Attachment, attachment_profile);

        Self {
            actor,
            operation: DirectorOperation::Supervised,
            target: Vec::new(),
            frame_state,
            boom_state,
            arm_state,
            attachment_state,
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("vehicle director")
    }

    async fn setup(&mut self) {
        info!("Vehicle director is running in {} mode", self.operation);
    }

    async fn wait_io_sub(&mut self, command_tx: CommandSender, mut signal_rx: SignalReceiver) {
        if let Ok(signal) = signal_rx.recv().await {
            match signal {
                Object::Rotator(rotator) => {
                    match rotator.source {
                        ENCODER_FRAME => {
                            self.actor.set_relative_rotation("frame", rotator.rotator);
                        }
                        ENCODER_BOOM => {
                            self.actor.set_relative_rotation("boom", rotator.rotator);
                        }
                        ENCODER_ARM => {
                            self.actor.set_relative_rotation("arm", rotator.rotator);
                        }
                        ENCODER_ATTACHMENT => {
                            self.actor
                                .set_relative_rotation("attachment", rotator.rotator);
                        }
                        _ => {}
                    }

                    if self.operation == DirectorOperation::Supervised
                        || self.operation == DirectorOperation::Autonomous
                    {
                        // TODO: Invoke supervisor
                    }

                    if self.operation == DirectorOperation::Autonomous {
                        // TODO: Invoke the planner

                        // Controller
                        {
                            let frame_error = 0.0;
                            let boom_error = 0.0;
                            let arm_error = 0.0;
                            let attachment_error = 0.0;

                            let mut motion = vec![];

                            if let Some(event) = self.frame_state.update(Some(frame_error)) {
                                log::debug!(
                                    "{:?} error: {}, value: {}",
                                    event.actuator,
                                    event.error,
                                    event.value
                                );

                                motion.push((event.actuator, event.value));
                            }

                            if let Some(event) = self.boom_state.update(Some(boom_error)) {
                                log::debug!(
                                    "{:?} error: {}, value: {}",
                                    event.actuator,
                                    event.error,
                                    event.value
                                );

                                motion.push((event.actuator, event.value));
                            }

                            if let Some(event) = self.arm_state.update(Some(arm_error)) {
                                log::debug!(
                                    "{:?} error: {}, value: {}",
                                    event.actuator,
                                    event.error,
                                    event.value
                                );

                                motion.push((event.actuator, event.value));
                            }

                            if let Some(event) =
                                self.attachment_state.update(Some(attachment_error))
                            {
                                log::debug!(
                                    "{:?} error: {}, value: {}",
                                    event.actuator,
                                    event.error,
                                    event.value
                                );

                                motion.push((event.actuator, event.value));
                            }

                            if !motion.is_empty() {
                                let motion_command = Motion::from_iter(motion);
                                if let Err(e) = command_tx.send(Object::Motion(motion_command)) {
                                    log::error!("Failed to send motion command: {}", e);
                                }
                            } else {
                                let motion_command = Motion::StopAll;
                                if let Err(e) = command_tx.send(Object::Motion(motion_command)) {
                                    log::error!("Failed to send motion command: {}", e);
                                }
                            }
                        }
                    }
                }
                Object::Engine(engine) => {
                    let in_emergency = false;
                    if in_emergency && engine.is_running() {
                        Self::command_emergency_stop(&command_tx);
                    }
                }
                Object::Target(target) => {
                    self.target.push(target);
                }
                _ => {}
            }
        }

        // log::trace!("Frame encoder: {}", value);

        // log::trace!(
        //     "Frame: Roll={:.2} Pitch={:.2} Yaw={:.2}",
        //     rotator.euler_angles().0.to_degrees(),
        //     rotator.euler_angles().1.to_degrees(),
        //     rotator.euler_angles().2.to_degrees()
        // );

        // let body_world_location = self.actor.world_location("frame");
        // trace!(
        //     "Frame: X={:.2} Y={:.2} Z={:.2}",
        //     body_world_location.x,
        //     body_world_location.y,
        //     body_world_location.z
        // );

        // let boom_world_location = self.actor.world_location("boom");
        // trace!(
        //     "Boom: X={:.2} Y={:.2} Z={:.2}",
        //     boom_world_location.x,
        //     boom_world_location.y,
        //     boom_world_location.z
        // );

        // let arm_world_location = self.actor.world_location("arm");
        // trace!(
        //     "Arm: X={:.2} Y={:.2} Z={:.2}",
        //     arm_world_location.x,
        //     arm_world_location.y,
        //     arm_world_location.z
        // );

        // let bucket_world_location = self.actor.world_location("attachment");
        // trace!(
        //     "Attachment: X={:.2} Y={:.2} Z={:.2}",
        //     bucket_world_location.x,
        //     bucket_world_location.y,
        //     bucket_world_location.z
        // );
    }
}
