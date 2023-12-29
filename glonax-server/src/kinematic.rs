use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};
use nalgebra::{Matrix4, Point3, Rotation3, Translation3, Vector3};

struct Actor {
    segments: Vec<(String, ActorSegment)>,
}

impl Actor {
    fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    fn attach_segment(&mut self, name: impl ToString, segment: ActorSegment) {
        self.segments.push((name.to_string(), segment));
    }

    fn location(&self) -> Translation3<f32> {
        self.segments[0].1.isometry.translation
    }

    fn rotation(&self) -> Rotation3<f32> {
        self.segments[0].1.isometry.rotation
    }

    fn set_location(&mut self, location: Vector3<f32>) {
        self.segments[0].1.isometry.translation = Translation3::from(location);
    }

    fn set_rotation(&mut self, rotation: Rotation3<f32>) {
        self.segments[0].1.isometry.rotation = rotation;
    }

    fn segment_location(&self, name: impl ToString) -> Point3<f32> {
        let mut transform = Matrix4::identity();

        for (sname, segment) in self.segments.iter() {
            transform *= segment.transformation();

            if sname == &name.to_string() {
                break;
            }
        }

        transform.transform_point(&Point3::new(0.0, 0.0, 0.0))
    }
}

struct ActorSegment {
    isometry: nalgebra::IsometryMatrix3<f32>,
}

impl ActorSegment {
    fn new(location: Vector3<f32>) -> Self {
        Self {
            isometry: nalgebra::IsometryMatrix3::from_parts(
                nalgebra::Translation3::from(location),
                nalgebra::Rotation3::identity(),
            ),
        }
    }

    fn location(&self) -> Translation3<f32> {
        self.isometry.translation
    }

    fn rotation(&self) -> Rotation3<f32> {
        self.isometry.rotation
    }

    fn transformation(&self) -> Matrix4<f32> {
        self.isometry.to_homogeneous()
    }
}

pub struct Kinematic;

impl<Cnf: Configurable> Component<Cnf> for Kinematic {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    // TODO: Calculate the forward kinematics
    // TODO: Store the forward kinematics in the context
    // TODO: Calculate the inverse kinematics, if there is a target
    // TODO: Store the inverse kinematics in the context, if there is a target
    // TODO: Store if target is reachable in the context, if there is a target
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        let mut robot = Actor::new();

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

        {
            let boom_world_location = robot.segment_location("boom");
            log::debug!("Boom world location: {:?}", boom_world_location);

            let bucket_world_location = robot.segment_location("bucket");
            log::debug!("Bucket world location: {:?}", bucket_world_location);
        }

        ///////////////

        let actor_world_location = Point3::from(robot.location().vector);

        // TODO: This is a world location, it has already been transformed
        let boom_world_location = Point3::new(0.0, 25.0, 140.0);
        let target = glonax::core::Target::from_point(300.0, 400.0, 330.0);

        let boom_length = 510.0;
        let arm_length = 310.0;

        let abs_target_distance = nalgebra::distance(&actor_world_location, &target.point);
        log::debug!("Absolute target distance: {}", abs_target_distance);

        let target_distance = nalgebra::distance(&boom_world_location, &target.point);
        log::debug!("Target distance: {}", target_distance);

        let theta0 = glonax::math::law_of_cosines(boom_length, arm_length, target_distance);
        log::debug!("Theta0: {}", theta0);

        let theta1 = glonax::math::law_of_cosines(boom_length, target_distance, arm_length);
        log::debug!("Theta1: {}", theta1);

        let arm_angle = -(std::f32::consts::PI - theta0);
        log::debug!("Arm angle: {}", arm_angle);

        // let yy = nalgebra::Rotation3::look_at_lh(&target.point.coords, &Vector3::y());
        // log::debug!("YY: {:?}", yy.euler_angles());

        let target_vector = target.point.coords - boom_world_location.coords;
        log::debug!("Target vector: {:?}", target_vector);

        // Target angle X: -10.4698925
        // Target angle Y: 21.585747
        // Target angle Z: -51.3402

        let target_angle = nalgebra::Rotation3::rotation_between(&target_vector, &Vector3::x());
        log::debug!(
            "Target angle X: {}",
            target_angle.unwrap().euler_angles().0.to_degrees()
        );
        log::debug!(
            "Target angle Y: {}",
            target_angle.unwrap().euler_angles().1.to_degrees()
        );
        log::debug!(
            "Target angle Z: {}",
            target_angle.unwrap().euler_angles().2.to_degrees()
        );

        // let qq = nalgebra::Rotation3::face_towards(&target.point.coords, &boom_location.coords);
        // log::debug!("QQ X: {}", qq.euler_angles().0.to_degrees());
        // log::debug!("QQ Y: {}", qq.euler_angles().1.to_degrees());
        // log::debug!("QQ Z: {}", qq.euler_angles().2.to_degrees());

        // let qq = nalgebra::Matrix4::look_at_lh(&boom_location, &target.point, &Vector3::y());
        // log::debug!("QQ: {:?}", qq);

        // ctx.map("forward_kinematic", enf_effector_pose);

        // println!("End effector pose: {:?}", enf_effector_pose);

        // let mut relative_error = nalgebra::Matrix4::zeros();
        // relative_error[glonax::core::Actuator::Slew as usize] = 24.4353;
        // relative_error[glonax::core::Actuator::Arm as usize] = 87.8354;
        // ctx.map("relative_error", relative_error);
    }
}
