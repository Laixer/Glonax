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
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut MachineState) {
        let point = Point3::new(1.0, 2.0, 3.0);

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
