use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, RobotState,
};
use nalgebra::{Rotation3, Translation3, Vector3};

#[derive(Default)]
pub struct KinematicComponent;

impl<Cnf: Configurable, R: RobotState> Component<Cnf, R> for KinematicComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn tick(&mut self, _ctx: &mut ComponentContext, runtime_state: &mut R) {
        let _pose = runtime_state.pose_mut();

        // Define a 2D point (you can extend this to 3D if needed)
        let _point = Vector3::new(1.0, 2.0, 0.0);

        // Translation
        let translation = Translation3::new(3.0, 4.0, 0.0);

        // Rotation (in radians)
        let angle = std::f32::consts::PI / 4.0; // 45 degrees
        let rotation = Rotation3::new(Vector3::z() * angle);

        // For scaling, you can directly use a diagonal matrix
        // let scale = Matrix3::new_nonuniform_scaling(&Vector3::new(1.5, 1.5, 1.0));

        // Combine them into an affine transformation matrix
        let _affine_transform = translation.to_homogeneous() * rotation.to_homogeneous();
        // * scale;

        // Apply the transformation
        // let transformed_point = affine_transform.transform_point(&point);

        // println!("Original point: {:?}", point);
        // println!("Transformed point: {:?}", transformed_point);

        // TODO: Calculate the forward kinematics
        // TODO: Store the forward kinematics in the pose
    }
}
