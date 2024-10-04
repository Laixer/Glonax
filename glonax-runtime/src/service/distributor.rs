use crate::{
    core::{MachineType, Object, Repository},
    global,
    runtime::{CommandSender, NullConfig, Service, ServiceContext, SignalReceiver},
};

pub struct Distributor {
    repository: Repository,
}

impl Service<NullConfig> for Distributor {
    fn new(_: NullConfig) -> Self
    where
        Self: Sized,
    {
        Self {
            repository: Repository::new(global::instance().clone(), MachineType::Excavator),
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("distributor")
    }

    async fn wait_io_sub(&mut self, _command_tx: CommandSender, mut signal_rx: SignalReceiver) {
        while let Ok(signal) = signal_rx.recv().await {
            match signal {
                Object::Engine(engine) => {
                    self.repository.engine = engine;
                }
                Object::Rotator(rotator) => {
                    self.repository.rotator.insert(rotator.source, rotator);
                    // debug!("Rotator size {}", self.repository.rotator.len());
                }
                Object::ModuleStatus(status) => {
                    self.repository
                        .module_status
                        .insert(status.name.clone(), status);
                    // debug!("Module status size {}", self.repository.module_status.len());
                }
                Object::Control(control) => {
                    self.repository.control.insert(control);
                    // debug!("Control size {}", self.repository.control.len());
                }
                _ => {}
            }
        }
    }
}
