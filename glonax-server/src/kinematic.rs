use glonax::{
    runtime::{Component, ComponentContext},
    RobotState,
};

#[derive(Default)]
pub struct KinematicComponent;

impl<R: RobotState> Component<R> for KinematicComponent {
    fn tick(&mut self, _ctx: &mut ComponentContext, runtime_state: &mut R) {
        let _pose = runtime_state.pose_mut();

        // TODO: Calculate the forward kinematics
        // TODO: Store the forward kinematics in the pose
    }
}
