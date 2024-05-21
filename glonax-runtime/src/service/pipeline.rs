use std::time::{Duration, Instant};

use crate::runtime::{
    CommandSender, Component, ComponentContext, IPCReceiver, InitComponent, PostComponent, Service,
    ServiceContext,
};

const COMPONENT_DELAY_THRESHOLD: Duration = Duration::from_micros(500);

pub struct Pipeline {
    ctx: ComponentContext,
    init_components: Vec<Box<dyn InitComponent<crate::runtime::NullConfig> + Send + Sync>>,
    tick_components: Vec<Box<dyn Component<crate::runtime::NullConfig> + Send + Sync>>,
    post_components: Vec<Box<dyn PostComponent<crate::runtime::NullConfig> + Send + Sync>>,
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
        self.tick_components.push(Box::new(component));
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

    pub fn add_init_component<C>(&mut self)
    where
        C: InitComponent<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        self.init_components
            .push(Box::new(C::new(crate::runtime::NullConfig {})));
    }

    pub fn add_post_component<C>(&mut self)
    where
        C: PostComponent<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        self.post_components
            .push(Box::new(C::new(crate::runtime::NullConfig {})));
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
            init_components: Vec::new(),
            tick_components: Vec::new(),
            post_components: Vec::new(),
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
    /// * `ipc_rx` - An `IPCReceiver` object representing the IPC receiver.
    /// * `command_tx` - A `CommandSender` object representing the command sender.
    fn tick(&mut self, ipc_rx: std::rc::Rc<IPCReceiver>, command_tx: CommandSender) {
        for (idx, component) in self.init_components.iter().enumerate() {
            let component_start = Instant::now();

            component.init(&mut self.ctx, ipc_rx.clone());

            if component_start.elapsed() > COMPONENT_DELAY_THRESHOLD {
                log::warn!("Init component {} is delaying execution", idx);
            }
        }

        for (idx, component) in self.tick_components.iter_mut().enumerate() {
            let component_start = Instant::now();

            component.tick(&mut self.ctx, command_tx.clone());

            if component_start.elapsed() > COMPONENT_DELAY_THRESHOLD {
                log::warn!("Tick component {} is delaying execution", idx);
            }
        }

        for (idx, component) in self.post_components.iter().enumerate() {
            let component_start = Instant::now();

            component.finalize(&mut self.ctx, command_tx.clone());

            if component_start.elapsed() > COMPONENT_DELAY_THRESHOLD {
                log::warn!("Post component {} is delaying execution", idx);
            }
        }

        self.ctx.post_tick();
    }
}
