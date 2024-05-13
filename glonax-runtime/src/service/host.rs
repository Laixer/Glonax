use sysinfo::{Components, System};

use crate::runtime::{Service, ServiceContext, SharedOperandState};

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

    async fn tick(&mut self, runtime_state: SharedOperandState) {
        self.system.refresh_memory();
        self.system.refresh_cpu();
        self.components.refresh();

        let load_avg = System::load_average();

        let mut runtime_state = runtime_state.write().await;
        runtime_state.state.vms.memory = (self.system.used_memory(), self.system.total_memory());
        runtime_state.state.vms.swap = (self.system.used_swap(), self.system.total_swap());
        runtime_state.state.vms.cpu_load = (load_avg.one, load_avg.five, load_avg.fifteen);
        runtime_state.state.vms.uptime = System::uptime();
        runtime_state.state.vms.timestamp = chrono::Utc::now();

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
