use std::time::Instant;

use crate::runtime::{
    CommandSender, Component, ComponentContext, Executor, IPCReceiver, InitComponent,
    PostComponent, SignalSender,
};

#[derive(Default)]
pub struct ComponentExecutor {
    ctx: ComponentContext,
    init_components: Vec<Box<dyn InitComponent<crate::runtime::NullConfig> + Send + Sync>>,
    tick_components: Vec<Box<dyn Component<crate::runtime::NullConfig> + Send + Sync>>,
    post_components: Vec<Box<dyn PostComponent<crate::runtime::NullConfig> + Send + Sync>>,
}

impl ComponentExecutor {
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

    /// Add an init component to the pipeline with the default configuration.
    ///
    /// This method will add an init component to the pipeline with the default configuration. The
    /// component will be provided with a copy of the runtime configuration.
    pub fn add_init_component<C>(&mut self)
    where
        C: InitComponent<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        self.init_components
            .push(Box::new(C::new(crate::runtime::NullConfig {})));
    }

    /// Add a post component to the pipeline with the default configuration.
    ///
    /// This method will add a post component to the pipeline with the default configuration. The
    /// component will be provided with a copy of the runtime configuration.
    pub fn add_post_component<C>(&mut self)
    where
        C: PostComponent<crate::runtime::NullConfig> + Send + Sync + 'static,
    {
        self.post_components
            .push(Box::new(C::new(crate::runtime::NullConfig {})));
    }
}

impl Executor for ComponentExecutor {
    /// Executes the tick operation for the `Pipeline`.
    ///
    /// # Arguments
    ///
    /// * `ipc_rx` - An `IPCReceiver` object representing the IPC receiver.
    /// * `command_tx` - A `CommandSender` object representing the command sender.
    fn run_init(&mut self, ipc_rx: std::rc::Rc<IPCReceiver>) {
        for (idx, component) in self.init_components.iter().enumerate() {
            let component_start = Instant::now();

            component.init(&mut self.ctx, ipc_rx.clone());

            if component_start.elapsed() > crate::consts::COMPONENT_DELAY_THRESHOLD {
                log::warn!("Init component {} is delaying execution", idx);
            }
        }
    }

    /// Executes the tick operation for the `Pipeline`.
    ///
    /// # Arguments
    ///
    /// * `ipc_rx` - An `IPCReceiver` object representing the IPC receiver.
    /// * `command_tx` - A `CommandSender` object representing the command sender.
    fn run_tick(&mut self) {
        for (idx, component) in self.tick_components.iter_mut().enumerate() {
            let component_start = Instant::now();

            component.tick(&mut self.ctx);

            if component_start.elapsed() > crate::consts::COMPONENT_DELAY_THRESHOLD {
                log::warn!("Tick component {} is delaying execution", idx);
            }
        }
    }

    /// Executes the tick operation for the `Pipeline`.
    ///
    /// # Arguments
    ///
    /// * `ipc_rx` - An `IPCReceiver` object representing the IPC receiver.
    /// * `command_tx` - A `CommandSender` object representing the command sender.
    fn run_post(&mut self, command_tx: CommandSender, signal_tx: std::rc::Rc<SignalSender>) {
        for (idx, component) in self.post_components.iter().enumerate() {
            let component_start = Instant::now();

            component.finalize(&mut self.ctx, command_tx.clone(), signal_tx.clone());

            if component_start.elapsed() > crate::consts::COMPONENT_DELAY_THRESHOLD {
                log::warn!("Post component {} is delaying execution", idx);
            }
        }

        self.ctx.post_tick();
    }
}
