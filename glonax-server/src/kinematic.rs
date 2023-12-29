use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};
use nalgebra::{Matrix4, Point3, Rotation3, Translation3, Vector3};

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
        let point = Point3::new(1.0, 2.0, 3.0);

        let actor_location = Point3::new(0.0, 0.0, 0.0);

        let boom_location = Point3::new(0.0, 25.0, 140.0);
        let target = glonax::core::Target::from_point(300.0, 400.0, 330.0);

        let boom_length = 510.0;
        let arm_length = 310.0;

        let abs_target_distance = nalgebra::distance(&actor_location, &target.point);
        log::debug!("Absolute target distance: {}", abs_target_distance);

        let target_distance = nalgebra::distance(&boom_location, &target.point);
        log::debug!("Target distance: {}", target_distance);

        let theta0 = glonax::math::law_of_cosines(boom_length, arm_length, target_distance);
        log::debug!("Theta0: {}", theta0);

        let theta1 = glonax::math::law_of_cosines(boom_length, target_distance, arm_length);
        log::debug!("Theta1: {}", theta1);

        let arm_angle = -(std::f32::consts::PI - theta0);
        log::debug!("Arm angle: {}", arm_angle);

        // let yy = nalgebra::Rotation3::look_at_lh(&target.point.coords, &Vector3::y());
        // log::debug!("YY: {:?}", yy.euler_angles());

        let target_vector = target.point.coords - boom_location.coords;
        log::debug!("Target vector: {:?}", target_vector);

        // arget angle X: -10.4698925
        // 16:37:43 [DEBUG] (1) Target angle Y: 21.585747
        // 16:37:43 [DEBUG] (1) Target angle Z: -51.3402
        

        let target_angle = nalgebra::Rotation3::rotation_between(&target_vector, &Vector3::x());
        log::debug!("Target angle X: {}", target_angle.unwrap().euler_angles().0.to_degrees());
        log::debug!("Target angle Y: {}", target_angle.unwrap().euler_angles().1.to_degrees());
        log::debug!("Target angle Z: {}", target_angle.unwrap().euler_angles().2.to_degrees());

        // let qq = nalgebra::Rotation3::face_towards(&target.point.coords, &boom_location.coords);
        // log::debug!("QQ X: {}", qq.euler_angles().0.to_degrees());
        // log::debug!("QQ Y: {}", qq.euler_angles().1.to_degrees());
        // log::debug!("QQ Z: {}", qq.euler_angles().2.to_degrees());

        // let qq = nalgebra::Matrix4::look_at_lh(&boom_location, &target.point, &Vector3::y());
        // log::debug!("QQ: {:?}", qq);

        // let rotation = nalgebra::Matrix3::<f32>::from_matrix_unchecked(nalgebra::Matrix4::identity());

        // qq.

        // nalgebra::Rotation::from

        // fn transformation_matrix(theta: f32, length: f32) -> Matrix4<f32> {
        //     // The order is Transform, Rotate, Scale
        //     let translation = Matrix4::new_translation(&Vector3::new(length, 0.0, 0.0));
        //     let rotation = Matrix4::new_rotation(Vector3::new(0.0, 0.0, theta));
        //     translation * rotation
        // }

        fn transformation_matrix(theta: f32, length: f32) -> Matrix4<f32> {
            // The order is Transform, Rotate, Scale

            // Rotation (in radians)
            let rotation = Rotation3::new(Vector3::z() * theta);

            // Translation
            let translation = Translation3::new(length, 0.0, 0.0);

            // Scale
            let scale = Matrix4::new_nonuniform_scaling(&Vector3::new(1.0, 1.0, 1.0));

            translation.to_homogeneous() * rotation.to_homogeneous() * scale
        }

        fn forward_kinematics(joint_angles: Vec<f32>, link_lengths: Vec<f32>) -> Matrix4<f32> {
            let mut t = Matrix4::identity();

            for (theta, length) in joint_angles.iter().zip(link_lengths.iter()) {
                t *= transformation_matrix(*theta, *length);
            }

            t
        }

        let joint_angles = [
            std::f32::consts::PI / 2.0,
            std::f32::consts::PI / 4.0,
            -std::f32::consts::PI / 6.0,
        ];
        let link_lengths = [6.0, 2.97, 1.5];

        let enf_effector_pose = forward_kinematics(joint_angles.to_vec(), link_lengths.to_vec());

        ctx.map("forward_kinematic", enf_effector_pose);

        // println!("End effector pose: {:?}", enf_effector_pose);

        let _p = enf_effector_pose.transform_point(&point);

        // println!("Transformed point: {:?}", p);

        // let mut relative_error = nalgebra::Matrix4::zeros();
        // relative_error[glonax::core::Actuator::Slew as usize] = 24.4353;
        // relative_error[glonax::core::Actuator::Arm as usize] = 87.8354;
        // ctx.map("relative_error", relative_error);
    }
}
