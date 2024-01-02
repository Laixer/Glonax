use glonax::{
    robot::{Actor, ActorSegment},
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};
use nalgebra::{Point3, Vector3};

pub struct Kinematic;

impl<Cnf: Configurable> Component<Cnf> for Kinematic {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    // TODO: Calculate the forward kinematics from encoders
    // TODO: Store the forward kinematics in the context
    // TODO: Calculate the inverse kinematics, if there is a target
    // TODO: Store the inverse kinematics in the context, if there is a target
    // TODO: Store if target is reachable in the context, if there is a target
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        // TODO: Add the robot to the context
        let mut robot = Actor::default();

        robot.attach_segment(
            "undercarriage",
            ActorSegment::new(Vector3::new(0.0, 0.0, 0.0)),
        );
        robot.attach_segment("body", ActorSegment::new(Vector3::new(-4.0, 5.0, 107.0)));
        robot.attach_segment("boom", ActorSegment::new(Vector3::new(4.0, 20.0, 33.0)));
        robot.attach_segment("arm", ActorSegment::new(Vector3::new(510.0, 20.0, 5.0)));
        robot.attach_segment(
            "bucket",
            ActorSegment::new(Vector3::new(310.0, -35.0, 45.0)),
        );

        // robot.set_location(Vector3::new(80.0, 0.0, 0.0));

        // {
        //     let boom_world_location = robot.segment_location("boom");
        //     log::debug!("Boom world location: {:?}", boom_world_location);

        //     let bucket_world_location = robot.segment_location("bucket");
        //     log::debug!("Bucket world location: {:?}", bucket_world_location);
        // }

        /////////////// IF THERE IS A TARGET ///////////////

        let actor_world_location = Point3::from(robot.location().vector);

        // TODO: This is a world location, it has already been transformed by the forward kinematics
        let boom_world_location = Point3::new(0.0, 25.0, 140.0);
        // TODO: This is given by the machine state
        let target = glonax::core::Target::from_point(300.0, 400.0, 330.0);

        // TODO: Can ask this from the robot
        let boom_length = 510.0;
        // TODO: Can ask this from the robot
        let arm_length = 310.0;

        // let actor_target_distance = nalgebra::distance(&actor_world_location, &target.point);
        // log::debug!("Actor target distance: {}", actor_target_distance);

        let target_distance = nalgebra::distance(&boom_world_location, &target.point);
        log::debug!("Tri-Arm target distance: {}", target_distance);

        let theta0 = glonax::math::law_of_cosines(boom_length, arm_length, target_distance);
        log::debug!("Theta0: {}rad {}deg", theta0, theta0.to_degrees());

        let arm_angle = -(std::f32::consts::PI - theta0);
        log::debug!("Arm angle: {}rad {}deg", arm_angle, arm_angle.to_degrees());

        let theta1 = glonax::math::law_of_cosines(boom_length, target_distance, arm_length);
        log::debug!("Theta1: {}rad {}deg", theta1, theta1.to_degrees());

        let target_direction = target.point.coords - boom_world_location.coords;
        log::debug!("Target direction: {:?}", target_direction);

        let direction_norm = target_direction.normalize();
        // log::debug!("Direction normalized: {:?}", direction_norm);

        // Correct: X: 0.0 Y: 21.585747 Z: 51.3402

        //// MANUAL ////

        let pitch = direction_norm
            .z
            .atan2((direction_norm.x.powi(2) + direction_norm.y.powi(2)).sqrt());
        let yaw = direction_norm.y.atan2(direction_norm.x);

        log::debug!("Pitch: {}deg", pitch.to_degrees());
        log::debug!("Yaw: {}deg", yaw.to_degrees());

        let boom_angle = theta1 + pitch;
        log::debug!(
            "Boom angle: {}rad {}deg",
            boom_angle,
            boom_angle.to_degrees()
        );

        // ctx.map(
        //     glonax::core::Actuator::Slew as u16,
        //     std::f32::consts::FRAC_PI_2,
        // );
        // ctx.map(
        //     glonax::core::Actuator::Boom as u16,
        //     std::f32::consts::FRAC_PI_3,
        // );
    }
}
