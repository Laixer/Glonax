use sysinfo::System;

use crate::{
    runtime::{Component, ComponentContext},
    Configurable, MachineState,
};

#[derive(Default)]
pub struct Host {
    system: System,
}

impl<Cnf: Configurable> Component<Cnf> for Host {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn tick(&mut self, _ctx: &mut ComponentContext, state: &mut MachineState) {
        self.system.refresh_memory();
        self.system.refresh_cpu();

        let load_avg = System::load_average();

        state.vms.memory = (self.system.used_memory(), self.system.total_memory());
        state.vms.swap = (self.system.used_swap(), self.system.total_swap());
        state.vms.cpu_load = (load_avg.one, load_avg.five, load_avg.fifteen);
        state.vms.uptime = System::uptime();
        state.vms.timestamp = chrono::Utc::now();
    }
}
