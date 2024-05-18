use std::time::{Duration, Instant};

use crate::runtime::{
    CommandSender, Component, ComponentContext, Service, ServiceContext, SharedOperandState,
};

pub struct Pipeline {
    ctx: ComponentContext,
    components: Vec<Box<dyn Component<crate::runtime::NullConfig> + Send + Sync>>,
}

impl Pipeline {
    /// Add a component to the pipeline.
    ///
    /// This method will add a component to the pipeline. The component will be provided with a copy
    /// of the runtime configuration.

    pub fn add_component<C>(&mut self, component: C)
    where
        C: Component<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        self.components.push(Box::new(component));
    }

    /// Add a component to the pipeline with the default configuration.
    ///
    /// This method will add a component to the pipeline with the default configuration. The component
    /// will be provided with a copy of the runtime configuration.
    pub fn add_component_default<C>(&mut self)
    where
        C: Component<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        self.add_component(C::new(crate::runtime::NullConfig {}));
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
            components: Vec::new(),
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

        for (idx, component) in self.components.iter_mut().enumerate() {
            let component_tick_start = Instant::now();

            component.tick(&mut self.ctx, machine_state, command_tx.clone());

            if component_tick_start.elapsed() > Duration::from_millis(2) {
                log::warn!("Component {} is delaying execution", idx);
            }
        }

        self.ctx.post_tick();
    }
}
