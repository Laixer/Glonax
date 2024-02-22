use sysinfo::System;

use crate::runtime::{Service, SharedOperandState};

pub struct Host {
    system: System,
}

impl<Cnf> Service<Cnf> for Host {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        log::debug!("Starting host component");

        Self {
            system: System::new_all(),
        }
    }

    fn tick(&mut self, runtime_state: SharedOperandState) {
        self.system.refresh_memory();
        self.system.refresh_cpu();

        let load_avg = System::load_average();

        if let Ok(mut runtime_state) = runtime_state.try_write() {
            runtime_state.state.vms.memory =
                (self.system.used_memory(), self.system.total_memory());
            runtime_state.state.vms.swap = (self.system.used_swap(), self.system.total_swap());
            runtime_state.state.vms.cpu_load = (load_avg.one, load_avg.five, load_avg.fifteen);
            runtime_state.state.vms.uptime = System::uptime();
            runtime_state.state.vms.timestamp = chrono::Utc::now();
        }
    }
}
