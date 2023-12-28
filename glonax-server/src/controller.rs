use glonax::{
    runtime::{Component, ComponentContext},
    Configurable, RobotState,
};

#[derive(Default)]
pub struct ControllerComponent;

impl<Cnf: Configurable, R: RobotState> Component<Cnf, R> for ControllerComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn tick(&mut self, ctx: &mut ComponentContext, _state: &mut R) {
        // let _pose = runtime_state.pose_mut();

        if let Some(_target) = ctx.target {
            // TODO: Calculate the inverse kinematics
            // TODO: Store the inverse kinematics in the pose
            // TODO: Translate resulting error into a control signal
            // TODO: Control signal to motion via motion profile

            // glonax::math::linear_motion(delta, lower_bound, offset, scale, inverse)

            // ctx.commit(glonax::core::Motion::StopAll);
        }
    }
}