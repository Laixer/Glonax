use sysinfo::{Components, System};

use crate::runtime::{Service, SharedOperandState};

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct HostConfig {
    // Host update interval.
    #[serde(default = "HostConfig::default_interval")]
    pub interval: u64,
}

impl HostConfig {
    fn default_interval() -> u64 {
        500
    }
}

pub struct Host {
    system: System,
    components: Components,
}

impl Service<HostConfig> for Host {
    fn new(_config: HostConfig) -> Self
    where
        Self: Sized,
    {
        Self {
            system: System::new_all(),
            components: Components::new_with_refreshed_list(),
        }
    }

    fn ctx(&self) -> crate::runtime::ServiceContext {
        crate::runtime::ServiceContext::new("host", Option::<String>::None)
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
