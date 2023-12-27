use std::collections::BTreeMap;

use glonax::{
    runtime::{Component, ComponentContext},
    RobotState,
};

pub struct PipelineComponent<R> {
    map: BTreeMap<i32, Box<dyn Component<R>>>,
}

impl<R> PipelineComponent<R> {
    pub fn new(components: Vec<(i32, Box<dyn Component<R>>)>) -> Self {
        let mut map = BTreeMap::new();

        for (order, component) in components {
            map.insert(order, component);
        }

        Self { map }
    }
}

impl<R> PipelineComponent<R> {
    pub fn make<C>(order: i32) -> (i32, Box<dyn Component<R>>)
    where
        C: Component<R> + Default + Send + Sync + 'static,
        R: RobotState + Send + Sync + 'static,
    {
        (order, Box::<C>::default())
    }
}

impl<R: glonax::RobotState> Component<R> for PipelineComponent<R> {
    fn tick(&mut self, _ctx: &mut ComponentContext, runtime_state: &mut R) {
        for service in self.map.values_mut() {
            service.tick(_ctx, runtime_state);
        }
    }
}
