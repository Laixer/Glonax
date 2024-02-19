use std::collections::BTreeMap;

use crate::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

pub struct Pipeline<Cnf> {
    map: BTreeMap<i32, Box<dyn Component<Cnf>>>,
}

impl<Cnf> Pipeline<Cnf> {
    pub fn new(components: Vec<(i32, Box<dyn Component<Cnf>>)>) -> Self {
        let mut map = BTreeMap::new();

        for (order, component) in components {
            map.insert(order, component);
        }

        Self { map }
    }
}

impl<Cnf: Configurable> Component<Cnf> for Pipeline<Cnf> {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn once(&mut self, ctx: &mut ComponentContext, _runtime_state: &mut MachineState) {
        for service in self.map.values_mut() {
            service.once(ctx, _runtime_state);
        }
    }

    fn tick(&mut self, ctx: &mut ComponentContext, runtime_state: &mut MachineState) {
        for service in self.map.values_mut() {
            service.tick(ctx, runtime_state);
        }
    }
}
