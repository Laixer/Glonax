use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};
use nalgebra::{Matrix4, Point3, Vector3};

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
        // let point = Point3::new(1.0, 2.0, 3.0);

        let actor_location = Point3::new(0.0, 0.0, 0.0);

        // The order is Transform, Rotate, Scale
        fn transformation(rotation: Vector3<f32>, translation: Vector3<f32>) -> Matrix4<f32> {
            let translation = Matrix4::new_translation(&translation);
            let rotation = Matrix4::new_rotation(rotation);
            translation * rotation
        }

        {
            let undercarriage_location = Vector3::new(0.0, 0.0, 0.0);
            let body_location = Vector3::new(-4.0, 5.0, 107.0);
            let boom_location = Vector3::new(4.0, 20.0, 33.0);
            let arm_location = Vector3::new(510.0, 20.0, 5.0);
            let bucket_location = Vector3::new(310.0, -35.0, 45.0);

            let mut t = Matrix4::identity();
            t *= transformation(Vector3::zeros(), undercarriage_location);
            t *= transformation(Vector3::zeros(), body_location);
            t *= transformation(Vector3::zeros(), boom_location);
            t *= transformation(Vector3::zeros(), arm_location);
            t *= transformation(Vector3::zeros(), bucket_location);

            let p = t.transform_point(&Point3::new(0.0, 0.0, 0.0));

            log::debug!("End effector point: {:?}", p);
        }

        // TODO: This is a world location, it has already been transformed
        let boom_world_location = Point3::new(0.0, 25.0, 140.0);
        let target = glonax::core::Target::from_point(300.0, 400.0, 330.0);

        let boom_length = 510.0;
        let arm_length = 310.0;

        let abs_target_distance = nalgebra::distance(&actor_location, &target.point);
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

        // arget angle X: -10.4698925
        // 16:37:43 [DEBUG] (1) Target angle Y: 21.585747
        // 16:37:43 [DEBUG] (1) Target angle Z: -51.3402

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
