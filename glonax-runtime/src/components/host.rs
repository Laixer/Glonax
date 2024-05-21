use sysinfo::System;

use crate::runtime::{Component, ComponentContext};

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

    fn tick(&mut self, ctx: &mut ComponentContext) {
        if ctx.iteration() % 50 != 0 {
            return;
        }

        // TODO: Call to external system, execution is non-deterministic. Test show significant delays.
        self.system.refresh_memory();
        self.system.refresh_cpu();

        let load_avg = System::load_average();

        let vms_signal = crate::core::Host {
            memory: (self.system.used_memory(), self.system.total_memory()),
            swap: (self.system.used_swap(), self.system.total_swap()),
            cpu_load: (load_avg.one, load_avg.five, load_avg.fifteen),
            uptime: System::uptime(),
            timestamp: chrono::Utc::now(),
            status: crate::core::HostStatus::Nominal,
        };

        ctx.machine.vms_signal = vms_signal;
        ctx.machine.vms_signal_instant = Some(std::time::Instant::now());
    }
}
