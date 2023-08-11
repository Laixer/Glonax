use sysinfo::{System, SystemExt};

use crate::core::{Metric, Signal};

const SIGNAL_FUNCTION_MEMORY: u32 = 0x17E;
const SIGNAL_FUNCTION_SWAP: u32 = 0x17F;
const SIGNAL_FUNCTION_CPU_1: u32 = 0x251;
const SIGNAL_FUNCTION_CPU_5: u32 = 0x252;
const SIGNAL_FUNCTION_CPU_15: u32 = 0x253;
const SIGNAL_FUNCTION_TIMESTAMP: u32 = 0x1A4;
const SIGNAL_FUNCTION_UPTIME: u32 = 0x1A5;

pub enum HostMessage {
    Memory(i32),
    Swap(i32),
    Cpu1(i32),
    Cpu5(i32),
    Cpu15(i32),
    Timestamp(u64),
    Uptime(u64),
}

impl From<crate::core::Signal> for HostMessage {
    fn from(value: crate::core::Signal) -> Self {
        match value.function {
            SIGNAL_FUNCTION_MEMORY => {
                HostMessage::Memory(if let crate::core::Metric::Percent(value) = value.metric {
                    value
                } else {
                    panic!("Invalid metric")
                })
            }
            SIGNAL_FUNCTION_SWAP => {
                HostMessage::Swap(if let crate::core::Metric::Percent(value) = value.metric {
                    value
                } else {
                    panic!("Invalid metric")
                })
            }
            SIGNAL_FUNCTION_CPU_1 => {
                HostMessage::Cpu1(if let crate::core::Metric::Percent(value) = value.metric {
                    value
                } else {
                    panic!("Invalid metric")
                })
            }
            SIGNAL_FUNCTION_CPU_5 => {
                HostMessage::Cpu5(if let crate::core::Metric::Percent(value) = value.metric {
                    value
                } else {
                    panic!("Invalid metric")
                })
            }
            SIGNAL_FUNCTION_CPU_15 => {
                HostMessage::Cpu15(if let crate::core::Metric::Percent(value) = value.metric {
                    value
                } else {
                    panic!("Invalid metric")
                })
            }
            SIGNAL_FUNCTION_TIMESTAMP => HostMessage::Timestamp(
                if let crate::core::Metric::Timestamp(value) = value.metric {
                    value
                } else {
                    panic!("Invalid metric")
                },
            ),
            SIGNAL_FUNCTION_UPTIME => {
                HostMessage::Uptime(if let crate::core::Metric::Count(value) = value.metric {
                    value
                } else {
                    panic!("Invalid metric")
                })
            }
            _ => panic!("Invalid function"),
        }
    }
}

impl std::fmt::Display for HostMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostMessage::Memory(metric) => write!(f, "Host memory: {}%", metric),
            HostMessage::Swap(metric) => write!(f, "Host swap: {}%", metric),
            HostMessage::Cpu1(metric) => write!(f, "Host CPU 1: {}", metric),
            HostMessage::Cpu5(metric) => write!(f, "Host CPU 5: {}", metric),
            HostMessage::Cpu15(metric) => write!(f, "Host CPU 15: {}", metric),
            HostMessage::Timestamp(metric) => write!(f, "Host timestamp: {}", metric),
            HostMessage::Uptime(metric) => write!(f, "Host uptime: {}", metric),
        }
    }
}

pub struct HostService {
    system: System,
    /// Node ID.
    node: u32,
}

impl HostService {
    pub fn new(node: u32) -> Self {
        let sys = System::new();

        Self { system: sys, node }
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
    fn collect_signals(&self, signals: &mut Vec<crate::core::Signal>) {
        signals.push(Signal::new(
            self.node,
            SIGNAL_FUNCTION_MEMORY,
            Metric::Percent(self.memory_used() as i32),
        ));
        signals.push(Signal::new(
            self.node,
            SIGNAL_FUNCTION_SWAP,
            Metric::Percent(self.swap_used() as i32),
        ));
        if self.uptime() % 15 == 0 {
            signals.push(Signal::new(
                self.node,
                SIGNAL_FUNCTION_UPTIME,
                Metric::Count(self.uptime()),
            ));
        }
        if self.uptime() % 60 == 0 {
            signals.push(Signal::new(
                self.node,
                SIGNAL_FUNCTION_TIMESTAMP,
                Metric::Timestamp(crate::core::time::now().as_secs()),
            ));
        }
        let load_avg = self.load_avg();
        signals.push(Signal::new(
            self.node,
            SIGNAL_FUNCTION_CPU_1,
            Metric::Percent(load_avg.0 as i32),
        ));
        signals.push(Signal::new(
            self.node,
            SIGNAL_FUNCTION_CPU_5,
            Metric::Percent(load_avg.1 as i32),
        ));
        signals.push(Signal::new(
            self.node,
            SIGNAL_FUNCTION_CPU_15,
            Metric::Percent(load_avg.2 as i32),
        ));
    }
}
