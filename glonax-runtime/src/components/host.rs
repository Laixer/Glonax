use sysinfo::System;

use crate::{
    runtime::{CommandSender, Component, ComponentContext},
    MachineState,
};

pub struct HostComponent {
    system: System,
}

impl<Cnf: Clone> Component<Cnf> for HostComponent {
    fn new(_config: Cnf) -> Self
    where
        Self: Sized,
    {
        Self {
            system: System::new_all(),
        }
    }

    fn tick(
        &mut self,
        ctx: &mut ComponentContext,
        state: &mut MachineState,
        _command_tx: CommandSender,
    ) {
        if ctx.iteration() % 20 != 0 {
            return;
        }

        self.system.refresh_memory();
        self.system.refresh_cpu();

        let load_avg = System::load_average();

        state.vms_signal_instant = Some(std::time::Instant::now());
        state.vms_signal.memory = (self.system.used_memory(), self.system.total_memory());
        state.vms_signal.swap = (self.system.used_swap(), self.system.total_swap());
        state.vms_signal.cpu_load = (load_avg.one, load_avg.five, load_avg.fifteen);
        state.vms_signal.uptime = System::uptime();
        state.vms_signal.timestamp = chrono::Utc::now();
    }
}
