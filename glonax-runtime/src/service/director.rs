use nalgebra::{Point3, Vector3};

use crate::{
    core::{Actuator, Control, Engine, Motion, Object},
    math::Linear,
    runtime::{CommandSender, NullConfig, Service, ServiceContext, SignalReceiver},
    world::{ActorBuilder, ActorSegment, World},
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

#[allow(dead_code)]
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
    world: World,
    operation: DirectorOperation,
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
        let mut world = World::default();

        // TODO: Build the actor from configuration and machine instance
        let actor = ActorBuilder::new(ROBOT_ACTOR_NAME)
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

        // TODO: Return weak reference to the actor
        world.add_actor(actor);

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
            world,
            operation: DirectorOperation::Supervised,
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
                    {
                        let actor = self.world.get_actor_by_name_mut(ROBOT_ACTOR_NAME).unwrap();

                        match rotator.source {
                            ENCODER_FRAME => {
                                actor.set_relative_rotation("frame", rotator.rotator);
                            }
                            ENCODER_BOOM => {
                                actor.set_relative_rotation("boom", rotator.rotator);
                            }
                            ENCODER_ARM => {
                                actor.set_relative_rotation("arm", rotator.rotator);
                            }
                            ENCODER_ATTACHMENT => {
                                actor.set_relative_rotation("attachment", rotator.rotator);
                            }
                            _ => {}
                        }
                    }

                    if self.operation == DirectorOperation::Supervised
                        || self.operation == DirectorOperation::Autonomous
                    {
                        // TODO: Invoke supervisor
                        // TODO: Supervisor should check:
                        // - If the actor is in a safe state (e.g. not in an emergency stop)
                        // - If the actor has all the necessary components (encoders, sensors, etc.)
                        // - If the actor is in a safe environment (e.g. not in a collision course)
                    }

                    if self.operation == DirectorOperation::Autonomous {
                        // Planner

                        // TODO: Before we call the planner, we need to check the supervisor status

                        let mut actuator_error = Vec::new();

                        {
                            let actor = self.world.get_actor_by_name(ROBOT_ACTOR_NAME).unwrap();
                            let target = self.world.get_actor_by_name("target0");

                            // TODO: Calculate this from the actor
                            const MAX_KINEMATIC_DISTANCE: f32 = 700.0;

                            {
                                let actor_world_distance = nalgebra::distance(
                                    &actor.location(),
                                    &Point3::new(0.0, 0.0, 0.0),
                                );
                                log::debug!("Actor origin distance: {:.2}", actor_world_distance);
                            }

                            if let Some(target) = target {
                                log::debug!("Objective target: {}", target.location());

                                let actor_target_distance =
                                    nalgebra::distance(&actor.location(), &target.location());
                                log::debug!("Actor target distance: {:.2}", actor_target_distance);

                                let boom_point = actor.relative_location("boom").unwrap();
                                let kinematic_target_distance = nalgebra::distance(
                                    &actor.location(),
                                    &(target.location() - boom_point.coords),
                                );
                                log::debug!(
                                    "Kinematic target distance: {:.2}",
                                    kinematic_target_distance
                                );

                                if kinematic_target_distance > MAX_KINEMATIC_DISTANCE {
                                    log::warn!("Target is out of reach");
                                }
                            }

                            if let Some(target) = target {
                                let boom_length = actor.relative_location("arm").unwrap().x;
                                // log::debug!("Boom length: {:?}", boom_length);

                                let arm_length = actor.relative_location("attachment").unwrap().x;
                                // log::debug!("Arm length: {:?}", arm_length);

                                let boom_world_location = actor.world_location("boom");

                                let target_distance =
                                    nalgebra::distance(&boom_world_location, &target.location());
                                log::debug!("Tri-Arm target distance: {:.2}", target_distance);

                                let target_direction = (target.location().coords
                                    - boom_world_location.coords)
                                    .normalize();

                                /////////////// SLEW YAW ANGLE ///////////////

                                let slew_angle = target_direction.y.atan2(target_direction.x);
                                log::debug!(
                                    "  Slew angle: {:.3}rad {:.2}deg",
                                    slew_angle,
                                    slew_angle.to_degrees()
                                );

                                // ctx.actuators.insert(crate::core::Actuator::Slew as u16, slew_angle);
                                actuator_error.push((Actuator::Slew, slew_angle));

                                /////////////// BOOM PITCH ANGLE ///////////////

                                let pitch = target_direction.z.atan2(
                                    (target_direction.x.powi(2) + target_direction.y.powi(2))
                                        .sqrt(),
                                );
                                // log::debug!("Pitch: {}deg", pitch.to_degrees());

                                let theta1 = crate::math::law_of_cosines(
                                    boom_length,
                                    target_distance,
                                    arm_length,
                                );
                                // log::debug!("Theta1: {}rad {}deg", theta1, theta1.to_degrees());

                                let boom_angle = theta1 + pitch;
                                log::debug!(
                                    "  Boom angle: {:.3}rad {:.2}deg",
                                    boom_angle,
                                    boom_angle.to_degrees()
                                );

                                // ctx.actuators.insert(crate::core::Actuator::Boom as u16, boom_angle);
                                actuator_error.push((Actuator::Boom, boom_angle));

                                /////////////// ARM PITCH ANGLE ///////////////

                                let theta0 = crate::math::law_of_cosines(
                                    boom_length,
                                    arm_length,
                                    target_distance,
                                );
                                // log::debug!("Theta0: {}rad {}deg", theta0, theta0.to_degrees());

                                let arm_angle = -(std::f32::consts::PI - theta0);
                                log::debug!(
                                    "  Arm angle: {:.3}rad {:.2}deg",
                                    arm_angle,
                                    arm_angle.to_degrees()
                                );

                                // ctx.actuators.insert(crate::core::Actuator::Arm as u16, arm_angle);
                                actuator_error.push((Actuator::Arm, arm_angle));
                            }
                        }

                        // Controller
                        {
                            let mut motion = vec![];

                            let frame_error = actuator_error
                                .iter()
                                .find(|(actuator, _)| *actuator == Actuator::Slew)
                                .map(|(_, error)| *error);

                            if let Some(event) = self.frame_state.update(frame_error) {
                                log::debug!(
                                    "{:?} error: {}, value: {}",
                                    event.actuator,
                                    event.error,
                                    event.value
                                );

                                motion.push((event.actuator, event.value));
                            }

                            let boom_error = actuator_error
                                .iter()
                                .find(|(actuator, _)| *actuator == Actuator::Boom)
                                .map(|(_, error)| *error);

                            if let Some(event) = self.boom_state.update(boom_error) {
                                log::debug!(
                                    "{:?} error: {}, value: {}",
                                    event.actuator,
                                    event.error,
                                    event.value
                                );

                                motion.push((event.actuator, event.value));
                            }

                            let arm_error = actuator_error
                                .iter()
                                .find(|(actuator, _)| *actuator == Actuator::Arm)
                                .map(|(_, error)| *error);

                            if let Some(event) = self.arm_state.update(arm_error) {
                                log::debug!(
                                    "{:?} error: {}, value: {}",
                                    event.actuator,
                                    event.error,
                                    event.value
                                );

                                motion.push((event.actuator, event.value));
                            }

                            let attachment_error = actuator_error
                                .iter()
                                .find(|(actuator, _)| *actuator == Actuator::Attachment)
                                .map(|(_, error)| *error);

                            if let Some(event) = self.attachment_state.update(attachment_error) {
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
                    if self.operation == DirectorOperation::Autonomous {
                        let actor = ActorBuilder::new("target0")
                            .with_location(target.point.coords)
                            .with_rotation(target.orientation.into())
                            .build();

                        self.world.add_actor(actor);
                    }
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
