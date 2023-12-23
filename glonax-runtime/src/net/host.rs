use sysinfo::System;

use crate::{runtime::SharedOperandState, RobotState};

pub struct HostService {
    system: System,
}

impl HostService {
    /// Refreshes the system information
    pub fn refresh(&mut self) {
        self.system.refresh_memory();
        self.system.refresh_cpu();
    }

    /// Returns the used memory in bytes
    #[inline]
    pub fn used_memory(&self) -> u64 {
        self.system.used_memory()
    }

    /// Returns the total memory in bytes
    #[inline]
    pub fn total_memory(&self) -> u64 {
        self.system.total_memory()
    }

    /// Returns the used swap in bytes
    #[inline]
    pub fn used_swap(&self) -> u64 {
        self.system.used_swap()
    }

    /// Returns the total swap in bytes
    #[inline]
    pub fn total_swap(&self) -> u64 {
        self.system.total_swap()
    }

    /// Returns the uptime in seconds
    #[inline]
    pub fn uptime(&self) -> u64 {
        System::uptime()
    }

    /// Returns the current timestamp
    #[inline]
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    /// Returns the CPU load
    #[inline]
    pub fn cpu_load(&self) -> (f64, f64, f64) {
        let load_avg = System::load_average();
        (load_avg.one, load_avg.five, load_avg.fifteen)
    }
}

impl Default for HostService {
    fn default() -> Self {
        let sys = System::new();

        Self { system: sys }
    }
}

impl HostService {
    pub async fn fill<R: RobotState>(&self, local_runtime_state: SharedOperandState<R>) {
        let mut runtime_state = local_runtime_state.write().await;

        runtime_state.state.vms_mut().memory = (self.used_memory(), self.total_memory());
        runtime_state.state.vms_mut().swap = (self.used_swap(), self.total_swap());
        runtime_state.state.vms_mut().cpu_load = self.cpu_load();
        runtime_state.state.vms_mut().uptime = self.uptime();
        runtime_state.state.vms_mut().timestamp = self.timestamp();
    }
}
