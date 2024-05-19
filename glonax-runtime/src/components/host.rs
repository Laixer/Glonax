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

        let vms_signal = crate::core::Host {
            memory: (self.system.used_memory(), self.system.total_memory()),
            swap: (self.system.used_swap(), self.system.total_swap()),
            cpu_load: (load_avg.one, load_avg.five, load_avg.fifteen),
            uptime: System::uptime(),
            timestamp: chrono::Utc::now(),
            status: crate::core::HostStatus::Nominal,
        };

        // TODO: state will not exist in the future
        // state.vms_signal_instant = Some(std::time::Instant::now());
        // state.vms_signal = vms_signal;

        let mut found = false;
        for signal in ctx.objects.iter_mut() {
            if let crate::core::Object::Host(vms_signal) = signal {
                *vms_signal = state.vms_signal;
                found = true;
                break;
            }
        }

        if !found {
            ctx.objects.push(crate::core::Object::Host(vms_signal));
        }
    }
}
