use std::collections::BTreeMap;

use crate::{
    runtime::{Component, ComponentContext},
    MachineState,
};

pub struct Pipeline<Cnf> {
    config: Cnf,
    _instance: crate::core::Instance,
    map: BTreeMap<i32, Box<dyn Component<Cnf>>>,
}

impl<Cnf> Pipeline<Cnf> {
    pub fn new(config: Cnf, instance: crate::core::Instance) -> Self {
        Self {
            config,
            _instance: instance,
            map: BTreeMap::new(),
        }
    }

    // TODO: Add instance to new
    /// Create a dynamic component with the given order.
    ///
    /// This method will create a dynamic component with the given order. The component will be
    /// provided with a copy of the runtime configuration.
    pub fn insert_component<C>(&mut self, order: i32)
    where
        C: Component<Cnf> + Send + Sync + 'static,
        Cnf: Clone,
    {
        self.map
            .insert(order, Box::new(C::new(self.config.clone())));
    }

    // TODO: Add instance to new
    /// Add a component to the pipeline.
    ///
    /// This method will add a component to the pipeline. The component will be provided with a copy
    /// of the runtime configuration.
    pub fn add_component<C>(&mut self, component: C)
    where
        C: Component<Cnf> + Send + Sync + 'static,
        Cnf: Clone,
    {
        let last_order = self.map.keys().last().unwrap_or(&0);

        self.map.insert(*last_order + 1, Box::new(component));
    }
}

// TODO: Replace with Service trait.
impl<Cnf: Clone> Component<Cnf> for Pipeline<Cnf> {
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
