use crate::{world::World, Machine};

use super::{CommandSender, IPCReceiver, SignalSender};

/// Component context.
///
/// The component context is provided to each component on each tick. The
/// component context is used to communicate within the component pipeline.
pub struct ComponentContext {
    /// Machine state.
    pub machine: Machine,
    /// World state.
    pub world: World,
    /// Actuator values.
    pub actuators: std::collections::HashMap<u16, f32>, // TODO: Find another way to pass actuator errors around. Maybe via objects.
    /// Last tick.
    last_tick: std::time::Instant,
    /// Iteration count.
    iteration: u64,
}

impl ComponentContext {
    /// Retrieve the tick delta.
    pub fn delta(&self) -> std::time::Duration {
        self.last_tick.elapsed()
    }

    /// Retrieve the iteration count.
    #[inline]
    pub fn iteration(&self) -> u64 {
        self.iteration
    }

    /// Called after all components are ticked.
    pub(crate) fn post_tick(&mut self) {
        self.actuators.clear();
        self.last_tick = std::time::Instant::now();
        self.iteration += 1;
    }
}

impl Default for ComponentContext {
    fn default() -> Self {
        Self {
            machine: Machine::default(),
            world: World::default(),
            actuators: std::collections::HashMap::new(),
            last_tick: std::time::Instant::now(),
            iteration: 0,
        }
    }
}

pub trait InitComponent<Cnf: Clone> {
    /// Construct a new component.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    /// Initialize the component.
    ///
    /// This method will be called on each tick of the runtime.
    /// How often the runtime ticks is determined by the runtime configuration.
    fn init(&self, ctx: &mut ComponentContext, ipc_rx: std::rc::Rc<IPCReceiver>);
}

pub trait PostComponent<Cnf: Clone> {
    /// Construct a new component.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    /// Finalize the component.
    ///
    /// This method will be called on each tick of the runtime.
    /// How often the runtime ticks is determined by the runtime configuration.
    fn finalize(
        &self,
        ctx: &mut ComponentContext,
        command_tx: CommandSender,
        signal_tx: std::rc::Rc<SignalSender>,
    );
}

pub trait Component<Cnf: Clone> {
    /// Construct a new component.
    ///
    /// This method will be called once on startup.
    /// The component should use this method to initialize itself.
    fn new(config: Cnf) -> Self
    where
        Self: Sized;

    /// Tick the component.
    ///
    /// This method will be called on each tick of the runtime.
    /// How often the runtime ticks is determined by the runtime configuration.
    fn tick(&mut self, ctx: &mut ComponentContext);
}
