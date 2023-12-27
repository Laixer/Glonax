use glonax::{
    runtime::{Component, ComponentContext},
    RobotState,
};

#[derive(Default)]
pub struct ControllerComponent;

impl<R: RobotState> Component<R> for ControllerComponent {
    fn tick(&mut self, ctx: &mut ComponentContext, state: &mut R) {
        // let _pose = runtime_state.pose_mut();

        if let Some(_target) = ctx.target {
            // TODO: Calculate the inverse kinematics
            // TODO: Store the inverse kinematics in the pose
            // TODO: Translate resulting error into a control signal
            // TODO: Control signal to motion via motion profile

            // ctx.commit(glonax::core::Motion::StopAll);
        }
    }
}
