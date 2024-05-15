use sysinfo::{Components, System};

use crate::runtime::{CommandSender, Service, ServiceContext, SharedOperandState};

pub struct Host {
    system: System,
    components: Components,
}

impl<C> Service<C> for Host {
    fn new(_config: C) -> Self
    where
        Self: Sized,
    {
        let system = System::new_all();

        if system.cpus().len() < 4 {
            log::warn!("System has less than 4 CPU cores");
        }

        Self {
            system,
            components: Components::new_with_refreshed_list(),
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("host")
    }

    async fn tick(&mut self, runtime_state: SharedOperandState, _command_tx: CommandSender) {
        self.system.refresh_memory();
        self.system.refresh_cpu();
        self.components.refresh();

        let load_avg = System::load_average();

        let mut runtime_state = runtime_state.write().await;
        runtime_state.state.vms_signal_instant = Some(std::time::Instant::now());
        runtime_state.state.vms_signal.memory =
            (self.system.used_memory(), self.system.total_memory());
        runtime_state.state.vms_signal.swap = (self.system.used_swap(), self.system.total_swap());
        runtime_state.state.vms_signal.cpu_load = (load_avg.one, load_avg.five, load_avg.fifteen);
        runtime_state.state.vms_signal.uptime = System::uptime();
        runtime_state.state.vms_signal.timestamp = chrono::Utc::now();

        for component in &self.components {
            if let Some(critical) = component.critical() {
                if component.temperature() > critical {
                    // TODO: Set system state to critical
                    log::warn!(
                        "{} is reaching cirital temperatures: {}°C",
                        component.label(),
                        component.temperature(),
                    );
                }
            }
        }
    }
}
