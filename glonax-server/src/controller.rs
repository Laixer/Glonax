use glonax::{
    runtime::{Component, ComponentContext},
    RobotState,
};

#[derive(Default)]
pub struct ControllerComponent;

impl<R: RobotState> Component<R> for ControllerComponent {
    fn tick(&mut self, _ctx: &mut ComponentContext, state: &mut R) {
        // let _pose = runtime_state.pose_mut();

        // TODO: Translate resulting error into a control signal
        // TODO: Control signal to motion via motion profile

        // ctx.motion_tx
        //     .try_send(glonax::core::Motion::StopAll)
        //     .unwrap();
    }
}
