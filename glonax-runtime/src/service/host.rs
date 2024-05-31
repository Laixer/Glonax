use std::time::Duration;

use sysinfo::{Components, System};

use crate::{
    core::Object,
    runtime::{Service, ServiceContext, SignalSender},
};

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

    async fn wait_io_pub(&mut self, signal_tx: SignalSender) {
        self.system.refresh_memory();
        self.system.refresh_cpu();
        self.components.refresh();

        let load_avg = System::load_average();

        let vms_signal = crate::core::Host {
            memory: (self.system.used_memory(), self.system.total_memory()),
            swap: (self.system.used_swap(), self.system.total_swap()),
            cpu_load: (load_avg.one, load_avg.five, load_avg.fifteen),
            uptime: System::uptime(),
            timestamp: chrono::Utc::now(),
            status: crate::core::HostStatus::Nominal,
        };

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

        if let Err(e) = signal_tx.send(Object::Host(vms_signal)) {
            log::error!("Failed to send host signal: {}", e);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
