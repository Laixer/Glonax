// Copyright (C) 2021 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

mod config;
mod device;
pub mod kernel;
mod runtime;

#[macro_use]
extern crate log;

pub use config::Config;
use device::{Hydraulic, IoDevice, MotionDevice};
use glonax_core::operand::Operand;

pub use runtime::{Runtime, RuntimeSettings};

use crate::device::{Composer, Device, Gamepad, MetricDevice};

/// Opaque runtime service for excavator kernel.
///
/// The excavator service uses the hydraulic device to control motion.
pub type ExcavatorService<'a> = RuntimeService<'a, Hydraulic, kernel::excavator::Excavator>;

/// Runtime service.
///
/// The runtime service is a convenient wrapper around the
/// runtime core code. It creates then configures the core
/// based on the global config and presents the caller with
/// a simple method to start the runtime loop.
pub struct RuntimeService<'a, M, K> {
    /// Current application configuration.
    config: &'a Config,
    /// Runtime core.
    runtime: Runtime<M, K>,
}

impl<'a, M: 'static, K> RuntimeService<'a, M, K>
where
    M: IoDevice + MotionDevice,
    K: Operand + 'static,
{
    /// Construct runtime service from configuration.
    pub fn from_config(config: &'a Config) -> Self {
        Self {
            config,
            runtime: Self::bootstrap(config),
        }
    }

    /// Create and probe the IO device.
    fn probe_io_device<D: IoDevice>(path: &String) -> std::sync::Arc<std::sync::Mutex<D>> {
        let mut io_device = D::from_path(path).unwrap();
        debug!("Probe '{}' device", io_device.name());
        io_device.probe();

        std::sync::Arc::new(std::sync::Mutex::new(io_device))
    }

    /// Create the runtime core.
    fn bootstrap(config: &'a Config) -> Runtime<M, K> {
        let motion_device = Self::probe_io_device::<M>(&config.motion_device);

        let program_queue = tokio::sync::mpsc::channel(1024);

        let mut rt = Runtime {
            operand: K::default(),
            motion_device: motion_device.clone(),
            event_bus: tokio::sync::mpsc::channel(64),
            program_queue: (program_queue.0, Some(program_queue.1)),
            settings: RuntimeSettings::from(config),
            task_pool: vec![],
            device_manager: runtime::DeviceManager::new(),
        };
        rt.device_manager.register_device(motion_device);
        rt
    }

    async fn config_services(&mut self) {
        if self.config.enable_term_shutdown {
            info!("Enable signals shutdown");

            let dispatcher = self.runtime.dispatch();

            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();

                info!("Termination requested");

                dispatcher.gracefull_shutdown().await.unwrap();
            });
        }

        if self.config.enable_autopilot {
            info!("Enable autopilot");

            // let mut imu = Inertial::new(SERIAL_INTERTIAL1)?;
            // log::info!("Name: {}", imu.name());
            // imu.probe();

            // let mut imu2 = Inertial::new(SERIAL_INTERTIAL2)?;
            // log::info!("Name: {}", imu2.name());
            // imu2.probe();

            let mut measure_compose = Composer::<Box<dyn MetricDevice + Send + Sync>>::new();
            debug!("Probe '{}' device", measure_compose.name());
            // measure_compose.insert(Box::new(imu));
            // measure_compose.insert(Box::new(imu2));
            measure_compose.probe();

            self.runtime.spawn_program_queue(measure_compose);
        }

        if self.config.enable_command {
            info!("Enable input device(s)");

            let mut gamepad = Gamepad::new();
            debug!("Probe '{}' device", gamepad.name());
            gamepad.probe();

            self.runtime.spawn_command_device(gamepad);
        }
    }

    /// Start the runtime service.
    ///
    /// This method consumes the runtime service.
    pub async fn launch(mut self) -> self::runtime::Result {
        self.config_services().await;

        self.runtime.program_queue.0.send(701).await.unwrap();

        self.runtime.run().await;

        Ok(())
    }
}
