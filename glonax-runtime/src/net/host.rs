use sysinfo::{System, SystemExt};

use crate::core::{Metric, Signal};

const DEVICE_NET_LOCAL_ADDR: u8 = 0x9E;

pub struct HostService {
    system: System,
}

impl HostService {
    pub fn new() -> Self {
        let sys = System::new();

        Self { system: sys }
    }

    /// Returns the CPU usage in percent
    pub fn memory_used(&self) -> f64 {
        (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0
    }

    /// Returns the CPU usage in percent
    pub fn swap_used(&self) -> f64 {
        (self.system.used_swap() as f64 / self.system.total_swap() as f64) * 100.0
    }

    /// Returns the system uptime in seconds
    pub fn uptime(&self) -> u64 {
        self.system.uptime()
    }

    /// Returns the system load average
    pub fn load_avg(&self) -> (f64, f64, f64) {
        let load_avg = self.system.load_average();
        (load_avg.one, load_avg.five, load_avg.fifteen)
    }

    /// Refreshes the system information
    pub fn refresh(&mut self) {
        self.system.refresh_memory();
        self.system.refresh_cpu();
    }
}

impl crate::channel::SignalSource for HostService {
    fn fetch2(&self, writer: &mut impl crate::channel::SignalChannel) {
        writer.push(Signal::new(
            DEVICE_NET_LOCAL_ADDR as u32,
            382,
            Metric::Percent(self.memory_used() as i32),
        ));
        writer.push(Signal::new(
            DEVICE_NET_LOCAL_ADDR as u32,
            383,
            Metric::Percent(self.swap_used() as i32),
        ));
        writer.push(Signal::new(
            DEVICE_NET_LOCAL_ADDR as u32,
            421,
            Metric::Count(self.uptime()),
        ));
        writer.push(Signal::new(
            DEVICE_NET_LOCAL_ADDR as u32,
            420,
            Metric::Timestamp(crate::core::time::now().as_secs()),
        ));
        let load_avg = self.load_avg();
        writer.push(Signal::new(
            DEVICE_NET_LOCAL_ADDR as u32,
            593,
            Metric::Percent(load_avg.0 as i32),
        ));
        writer.push(Signal::new(
            DEVICE_NET_LOCAL_ADDR as u32,
            594,
            Metric::Percent(load_avg.1 as i32),
        ));
        writer.push(Signal::new(
            DEVICE_NET_LOCAL_ADDR as u32,
            595,
            Metric::Percent(load_avg.2 as i32),
        ));
    }
}

impl crate::channel::BroadcastSource<crate::transport::Signal> for HostService {
    fn fetch(&self, writer: &crate::channel::BroadcastChannelWriter<crate::transport::Signal>) {
        writer
            .send(crate::transport::Signal::new(
                DEVICE_NET_LOCAL_ADDR as u32,
                382,
                crate::transport::signal::Metric::Percent(self.memory_used() as i32),
            ))
            .ok();
        writer
            .send(crate::transport::Signal::new(
                DEVICE_NET_LOCAL_ADDR as u32,
                383,
                crate::transport::signal::Metric::Percent(self.swap_used() as i32),
            ))
            .ok();
        let load_avg = self.load_avg();
        writer
            .send(crate::transport::Signal::new(
                DEVICE_NET_LOCAL_ADDR as u32,
                593,
                crate::transport::signal::Metric::Percent(load_avg.0 as i32),
            ))
            .ok();
        writer
            .send(crate::transport::Signal::new(
                DEVICE_NET_LOCAL_ADDR as u32,
                594,
                crate::transport::signal::Metric::Percent(load_avg.1 as i32),
            ))
            .ok();
        writer
            .send(crate::transport::Signal::new(
                DEVICE_NET_LOCAL_ADDR as u32,
                595,
                crate::transport::signal::Metric::Percent(load_avg.2 as i32),
            ))
            .ok();
    }
}
