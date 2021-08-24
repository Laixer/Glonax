use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    device::{CommandDevice, CommandEvent, MotionDevice},
    orchestrator::{MotionControl, NormalControl, ToMotionControl},
    Config,
};

pub struct ActuatorMap(std::collections::HashMap<u32, u32>);

impl ActuatorMap {
    /// Create new and empty ActuatorMap.
    pub fn new() -> Self {
        ActuatorMap(std::collections::HashMap::default())
    }

    /// Get the map value or return the input as default.
    pub fn get_or_default(&self, value: u32) -> u32 {
        self.0.get(&value).unwrap_or(&value).clone()
    }

    /// Insert mapping value.
    ///
    /// If the value was already in the map then its updated
    /// and the old value is returned. In all other cases
    /// `None` is returned.
    pub fn insert(&mut self, k: u32, v: u32) -> Option<u32> {
        self.0.insert(k, v)
    }

    /// Flip two actuators.
    ///
    /// After insert the key becomes the value and vice versa.
    /// This is the recommended way to map actuators because it
    /// is a non-reducing operation. All actuators will remain
    /// addressable.
    pub fn insert_bilateral(&mut self, k: u32, v: u32) {
        self.0.insert(k, v);
        self.0.insert(v, k);
    }
}

#[derive(Debug)]
pub enum RuntimeEvent {
    Motion(MotionControl),
    Shutdown,
}

unsafe impl Sync for RuntimeEvent {}
unsafe impl Send for RuntimeEvent {}

pub struct Dispatch(Sender<RuntimeEvent>);

impl Dispatch {
    // FUTURE: Maybe rename in the future
    /// Request motion.
    ///
    /// Post motion request on the runtime queue. This method will
    /// *not* wait until the action is executed.
    #[inline]
    async fn motion(
        &self,
        motion: impl ToMotionControl,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RuntimeEvent>> {
        self.0
            .send(RuntimeEvent::Motion(motion.to_motion_control()))
            .await
    }

    /// Request runtime shutdown.
    ///
    /// This is the recommended way to shutdown the runtime. Some
    /// subsystems may need time to close resources or dispose
    /// managed objects.
    ///
    /// This method will *not* wait until the action is executed.
    #[inline]
    pub async fn gracefull_shutdown(
        &self,
    ) -> Result<(), tokio::sync::mpsc::error::SendError<RuntimeEvent>> {
        self.0.send(RuntimeEvent::Shutdown).await
    }
}

// TODO: None of the fields should be pub.
pub struct RuntimeSettings {
    pub allow_program_motion: bool,
}

impl From<&Config> for RuntimeSettings {
    fn from(config: &Config) -> Self {
        Self {
            allow_program_motion: config.enable_autopilot,
        }
    }
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            allow_program_motion: true,
        }
    }
}

// TODO: None of the fields should be pub.
pub struct Runtime<A> {
    /// Actuator device.
    pub motion_device: A,
    /// Metric device.
    // pub metric_device: Option<M>, // TODO: If we're taking the device anyway then do not `own` this var.
    /// Optional actuator mapping.
    pub actuator_map: Option<ActuatorMap>,
    /// Runtime event bus.
    pub event_bus: (Sender<RuntimeEvent>, Receiver<RuntimeEvent>),
    /// Runtime settings.
    pub settings: RuntimeSettings,
    pub task_pool: Vec<JoinHandle<()>>,
}

impl<A> Runtime<A> {
    #[inline]
    pub fn dispatch(&self) -> Dispatch {
        Dispatch(self.event_bus.0.clone())
    }
}

impl<A: MotionDevice> Runtime<A> {
    /// Drive motion on an actuator.
    pub fn drive_motion(&mut self, motion: impl ToMotionControl) {
        let motion_control = motion.to_motion_control();

        // If the actuator is mapped to another value then
        // replace the incoming code with the mapped value.
        // In all other situations return the incoming code
        // the as default value.
        let actuator = match &self.actuator_map {
            Some(map) => map.get_or_default(motion_control.actuator),
            None => motion_control.actuator,
        };

        debug!("Move actuator {} with {}", actuator, motion_control.value);

        self.motion_device.actuate(actuator, motion_control.value);
    }

    pub fn spawn_command_device<C: CommandDevice + Send + 'static>(
        &mut self,
        mut command_device: C,
    ) -> &mut Self {
        let dispatcher = self.dispatch();

        let task_handle = tokio::task::spawn(async move {
            use crate::orchestrator::Actuator;

            // Map the gamepad scancodes to the actuators.
            const COMMAND_MAP: [(i16, Actuator); 6] = [
                (0, Actuator::Arm),
                (1, Actuator::Slew),
                (2, Actuator::Boom),
                (3, Actuator::Bucket),
                (5, Actuator::LimpLeft),
                (6, Actuator::LimpRight),
            ];
            loop {
                match command_device.next() {
                    Some(CommandEvent::DirectMotion { code, value }) => {
                        if let Some((_, actuator)) = COMMAND_MAP.iter().find(|(x, _)| x == &code) {
                            dispatcher
                                .motion(NormalControl {
                                    actuator: actuator.clone(),
                                    value,
                                    ..Default::default()
                                })
                                .await
                                .unwrap();
                        }
                    }
                    None => tokio::time::sleep(tokio::time::Duration::from_millis(5)).await,
                }
            }
        });

        self.task_pool.push(task_handle);
        self
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(event) = self.event_bus.1.recv().await {
                match event {
                    RuntimeEvent::Motion(motion_event) => self.drive_motion(motion_event),
                    RuntimeEvent::Shutdown => break,
                }
            };
        }

        // TODO: Cancel all async tasks.

        for handle in &self.task_pool {
            handle.abort()
        }
    }
}

impl<A: MotionDevice> Runtime<A> {
    pub fn spawn_program_queue<D, P>(
        &mut self,
        mut metric_devices: crate::device::Composer<Box<D>>,
        mut program: P,
    ) -> &mut Self
    where
        D: crate::device::MetricDevice + Send + Sync + 'static + ?Sized,
        P: crate::kernel::Program + Send + Sync + 'static,
        P::Motion: ToMotionControl + Send + Sync,
    {
        let dispatcher = self.dispatch();
        let allow_program_motion = self.settings.allow_program_motion;

        let task_handle = tokio::task::spawn(async move {
            while !program.can_terminate() {
                for (idx, device) in &mut metric_devices.iter_mut() {
                    match device.next() {
                        Some(value) => {
                            program.push(idx.clone(), value);
                        }
                        None => {}
                    }
                }

                if let Some(motion) = program.step() {
                    if allow_program_motion {
                        if let Err(_) = dispatcher.motion(motion).await {
                            warn!("Program terminated without completion");
                            return;
                        }
                    }
                }
            }

            if let Some(motion) = program.term_action() {
                if allow_program_motion {
                    if let Err(_) = dispatcher.motion(motion).await {
                        warn!("Program terminated without completion");
                        return;
                    }
                }
            }

            info!("Program terminated");
        });

        self.task_pool.push(task_handle);
        self
    }
}
