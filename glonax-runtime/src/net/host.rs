use sysinfo::{System, SystemExt};

use crate::core::{Metric, Signal};

pub struct HostService {
    system: System,
}

impl HostService {
    /// Creates a new host service
    pub fn new() -> Self {
        let sys = System::new();

        Self { system: sys }
    }

    /// Refreshes the system information
    pub fn refresh(&mut self) {
        self.system.refresh_memory();
        self.system.refresh_cpu();
    }
}

impl crate::channel::SignalSource for HostService {
    fn collect_signals(&self, signals: &mut Vec<crate::core::Signal>) {
        signals.push(Signal::new(Metric::VmsMemoryUsage((
            self.system.used_memory(),
            self.system.total_memory(),
        ))));
        signals.push(Signal::new(Metric::VmsSwapUsage((
            self.system.used_swap(),
            self.system.total_swap(),
        ))));
        if self.system.uptime() % 10 == 0 {
            signals.push(Signal::new(Metric::VmsUptime(self.system.uptime())));
        }
        if self.system.uptime() % 60 == 0 {
            signals.push(Signal::new(Metric::VmsTimestamp(chrono::Utc::now())));
        }
        let load_avg = self.system.load_average();
        signals.push(Signal::new(Metric::VmsCpuLoad((
            load_avg.one,
            load_avg.five,
            load_avg.fifteen,
        ))));
    }
}
