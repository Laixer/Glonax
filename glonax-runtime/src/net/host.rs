use sysinfo::{System, SystemExt};

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

    pub fn used_memory(&self) -> u64 {
        self.system.used_memory()
    }

    pub fn total_memory(&self) -> u64 {
        self.system.total_memory()
    }

    pub fn used_swap(&self) -> u64 {
        self.system.used_swap()
    }

    pub fn total_swap(&self) -> u64 {
        self.system.total_swap()
    }

    pub fn uptime(&self) -> u64 {
        self.system.uptime()
    }

    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    pub fn cpu_load(&self) -> (f64, f64, f64) {
        let load_avg = self.system.load_average();
        (load_avg.one, load_avg.five, load_avg.fifteen)
    }
}

impl HostService {
    pub async fn fill(&self, local_machine_state: crate::runtime::SharedMachineState) {
        let mut machine_state = local_machine_state.write().await;

        machine_state.state.vms.memory = (self.used_memory(), self.total_memory());
        machine_state.state.vms.swap = (self.used_swap(), self.total_swap());
        machine_state.state.vms.cpu_load = self.cpu_load();
        machine_state.state.vms.uptime = self.uptime();
        machine_state.state.vms.timestamp = self.timestamp();
    }
}
