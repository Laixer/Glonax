use std::collections::BTreeMap;

use crate::runtime::{
    CommandSender, Component, ComponentContext, Service, ServiceContext, SharedOperandState,
};

pub struct Pipeline {
    ctx: ComponentContext,
    map: BTreeMap<i32, Box<dyn Component<crate::runtime::NullConfig> + Send + Sync>>,
}

impl Pipeline {
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
    /// Creates a new instance of `Pipeline`.
    ///
    /// # Arguments
    ///
    /// * `_` - A `NullConfig` object (ignored).
    ///
    /// # Returns
    ///
    /// A new instance of `Pipeline`.
    fn new(_: crate::runtime::NullConfig) -> Self
    where
        Self: Sized,
    {
        Self {
            ctx: ComponentContext::default(),
            map: BTreeMap::new(),
        }
    }

    /// Returns the service context for the `Pipeline`.
    ///
    /// # Returns
    ///
    /// A `ServiceContext` object representing the service context for the `Pipeline`.
    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("pipeline")
    }

    /// Executes the tick operation for the `Pipeline`.
    ///
    /// # Arguments
    ///
    /// * `runtime_state` - A `SharedOperandState` object representing the runtime state.
    /// * `command_tx` - A `CommandSender` object representing the command sender.
    async fn tick(&mut self, runtime_state: SharedOperandState, command_tx: CommandSender) {
        let machine_state = &mut runtime_state.write().await.state;

        let loop_start = std::time::Instant::now();

        for service in self.map.values_mut() {
            service.tick(&mut self.ctx, machine_state, command_tx.clone());
        }

        let loop_duration = loop_start.elapsed();
        log::trace!("Control loop duration: {:?}", loop_duration);

        if loop_duration > std::time::Duration::from_millis(10) {
            log::warn!("Control loop delta is too high: {:?}", loop_duration);
        }

        self.ctx.post_tick();
    }
}
