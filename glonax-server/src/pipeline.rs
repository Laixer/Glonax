use glonax::{
    runtime::{Component, ComponentContext},
    RobotState,
};

#[derive(Default)]
pub struct PipelineComponent<R> {
    list: Vec<Box<dyn Component<R>>>,
}

impl<R> PipelineComponent<R> {
    pub fn add<C>(&mut self)
    where
        C: Component<R> + Default + Send + Sync + 'static,
        R: RobotState + Send + Sync + 'static,
    {
        self.list.push(Box::<C>::default());
    }
}

impl<R: glonax::RobotState> Component<R> for PipelineComponent<R> {
    fn tick(&mut self, _ctx: &mut ComponentContext, runtime_state: &mut R) {
        for service in self.list.iter_mut() {
            service.tick(_ctx, runtime_state);
        }
    }
}

//     let motion_tx = self.motion_tx.clone();

//     loop {
//         interval.tick().await;

//         // Collect all motion commands, send them
//         for motion in ctx.motion_queue {
//             motion_tx.send(motion).await.unwrap();
//         }
//     }
