use sysinfo::System;

use crate::{
    runtime::{Component, ComponentContext},
    Configurable, RobotState,
};

#[derive(Default)]
pub struct Host {
    system: System,
}

impl<Cnf: Configurable, R: RobotState> Component<Cnf, R> for Host {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn tick(&mut self, _ctx: &mut ComponentContext, runtime_state: &mut R) {
        let vms = runtime_state.vms_mut();

        self.system.refresh_memory();
        self.system.refresh_cpu();

        let load_avg = System::load_average();

        vms.memory = (self.system.used_memory(), self.system.total_memory());
        vms.swap = (self.system.used_swap(), self.system.total_swap());
        vms.cpu_load = (load_avg.one, load_avg.five, load_avg.fifteen);
        vms.uptime = System::uptime();
        vms.timestamp = chrono::Utc::now();
    }
}
