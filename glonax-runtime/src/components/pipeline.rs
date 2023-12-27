use std::collections::BTreeMap;

use crate::{
    runtime::{Component, ComponentContext},
    RobotState,
};

pub struct Pipeline<R> {
    map: BTreeMap<i32, Box<dyn Component<R>>>,
}

impl<R> Pipeline<R> {
    pub fn new(components: Vec<(i32, Box<dyn Component<R>>)>) -> Self {
        let mut map = BTreeMap::new();

        for (order, component) in components {
            map.insert(order, component);
        }

        Self { map }
    }
}

impl<R> Pipeline<R> {
    pub fn make<C>(order: i32) -> (i32, Box<dyn Component<R>>)
    where
        C: Component<R> + Default + Send + Sync + 'static,
        R: RobotState + Send + Sync + 'static,
    {
        (order, Box::<C>::default())
    }
}

impl<R: RobotState> Component<R> for Pipeline<R> {
    fn tick(&mut self, _ctx: &mut ComponentContext, runtime_state: &mut R) {
        for service in self.map.values_mut() {
            service.tick(_ctx, runtime_state);
        }
    }
}
