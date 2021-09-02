pub mod common;
mod config;
// mod daemon;
mod device;
pub mod ice;
pub mod kernel;
mod runtime;

#[macro_use]
extern crate log;

pub use config::Config;
pub use runtime::{Runtime, RuntimeSettings};

use crate::device::{Composer, Device, Gamepad, Hydraulic};

pub struct RuntimeService<'a, K> {
    config: &'a self::Config,
    runtime: Runtime<Hydraulic, K>,
}

impl<'a, K: kernel::excavator::Operand + 'static> RuntimeService<'a, K> {
    /// Construct runtime service from configuration.
    pub fn from_config(config: &'a Config) -> Self {
        Self {
            config,
            runtime: Self::bootstrap(config),
        }
    }

    fn bootstrap(config: &'a Config) -> Runtime<Hydraulic, K> {
        let mut hydraulic_motion = Hydraulic::new(&config.motion_device).unwrap();
        debug!("Probe '{}' device", hydraulic_motion.name());
        hydraulic_motion.probe();

        Runtime {
            operand: K::default(),
            motion_device: hydraulic_motion,
            event_bus: tokio::sync::mpsc::channel(128),
            settings: RuntimeSettings::from(config),
            task_pool: vec![],
        }
    }

    /// Start the runtime service.
    pub async fn rt_service(&mut self) {
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

            let mut measure_compose =
                Composer::<Box<dyn device::MetricDevice + Send + Sync>>::new();
            debug!("Probe '{}' device", measure_compose.name());
            // measure_compose.insert(Box::new(imu));
            // measure_compose.insert(Box::new(imu2));
            measure_compose.probe();

            // rt.spawn_program_queue(
            //     measure_compose,
            //     glonax::kernel::arm_balance::ArmBalanceProgram::new(),
            // );
            self.runtime
                .spawn_program_queue(measure_compose, kernel::drive::DriveProgram::new());
        }

        if self.config.enable_command {
            info!("Enable input device(s)");

            let mut gamepad = Gamepad::new();
            debug!("Probe '{}' device", gamepad.name());
            gamepad.probe();

            self.runtime.spawn_command_device(gamepad);
        }

        self.runtime.run().await
    }
}
