use glonax::RobotState;

use crate::state::{Component, ComponentContext};

#[derive(Default)]
pub struct KinematicComponent;

impl Component for KinematicComponent {
    fn tick<R: RobotState>(&mut self, _ctx: &mut ComponentContext, runtime_state: &mut R) {
        let _pose = runtime_state.pose_mut();

        // TODO: Calculate the forward kinematics
        // TODO: Store the forward kinematics in the pose
    }
}
