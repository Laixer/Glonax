use std::collections::BTreeMap;

use crate::{
    runtime::{Component, ComponentContext},
    Configurable, RobotState,
};

pub struct Pipeline<Cnf, R> {
    map: BTreeMap<i32, Box<dyn Component<Cnf, R>>>,
}

impl<Cnf, R> Pipeline<Cnf, R> {
    pub fn new(components: Vec<(i32, Box<dyn Component<Cnf, R>>)>) -> Self {
        let mut map = BTreeMap::new();

        for (order, component) in components {
            map.insert(order, component);
        }

        Self { map }
    }
}

impl<Cnf, R> Pipeline<Cnf, R> {
    pub fn make<C>(order: i32) -> (i32, Box<dyn Component<Cnf, R>>)
    where
        C: Component<Cnf, R> + Default + Send + Sync + 'static,
        Cnf: Configurable,
        R: RobotState + Send + Sync + 'static,
    {
        (order, Box::<C>::default())
    }
}

impl<Cnf: Configurable, R: RobotState> Component<Cnf, R> for Pipeline<Cnf, R> {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn tick(&mut self, _ctx: &mut ComponentContext, runtime_state: &mut R) {
        for service in self.map.values_mut() {
            service.tick(_ctx, runtime_state);
        }
    }
}
