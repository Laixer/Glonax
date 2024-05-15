use std::collections::BTreeMap;

use crate::runtime::{
    Component, ComponentContext, MotionSender, Service, ServiceContext, SharedOperandState,
};

pub struct Pipeline {
    ctx: ComponentContext,
    map: BTreeMap<i32, Box<dyn Component<crate::runtime::NullConfig> + Send + Sync>>,
}

impl Pipeline {
    /// Construct a new pipeline.
    pub fn new(command_tx: MotionSender) -> Self {
        Self {
            ctx: ComponentContext::new(command_tx),
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
        C: Component<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        self.map
            .insert(order, Box::new(C::new(crate::runtime::NullConfig {})));
    }

    // TODO: Add instance to new
    /// Add a component to the pipeline.
    ///
    /// This method will add a component to the pipeline. The component will be provided with a copy
    /// of the runtime configuration.
    pub fn add_component<C>(&mut self, component: C)
    where
        C: Component<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        let last_order = self.map.keys().last().unwrap_or(&0);

        self.map.insert(*last_order + 1, Box::new(component));
    }
}

impl Service<crate::runtime::NullConfig> for Pipeline {
    fn new(_: crate::runtime::NullConfig) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("pipeline")
    }

    async fn setup(&mut self, runtime_state: SharedOperandState) {
        let machine_state = &mut runtime_state.write().await.state;

        for service in self.map.values_mut() {
            service.once(&mut self.ctx, machine_state);
        }

        self.ctx.post_tick();
    }

    async fn teardown(&mut self, _runtime_state: SharedOperandState) {
        // TODO: Call teardown on each component

        self.ctx.post_tick();
    }

    async fn tick(&mut self, runtime_state: SharedOperandState, _command_tx: MotionSender) {
        let machine_state = &mut runtime_state.write().await.state;

        for service in self.map.values_mut() {
            service.tick(&mut self.ctx, machine_state);
        }

        self.ctx.post_tick();
    }
}
