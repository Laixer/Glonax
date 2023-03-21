// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};
use glonax::{
    core::{
        input::{ButtonState, Scancode},
        Level,
    },
    transport::{Motion, ToMotion},
};

mod config;
mod gamepad;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax input daemon", long_about = None)]
struct Args {
    /// Remote network address.
    #[arg(short = 'c', long = "connect", default_value = "http://[::1]:50051")]
    address: String,
    /// Gamepad input device.
    #[arg(value_hint = ValueHint::FilePath)]
    device: String,
    /// Daemonize the service.
    #[arg(long)]
    daemon: bool,
    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut config = config::InputConfig {
        address: args.address,
        device: args.device,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = env!("CARGO_BIN_NAME").to_string();
    config.global.daemon = args.daemon;

    let mut log_config = simplelog::ConfigBuilder::new();
    if args.daemon {
        log_config.set_time_level(log::LevelFilter::Off);
        log_config.set_thread_level(log::LevelFilter::Off);
    } else {
        log_config.set_time_offset_to_local().ok();
        log_config.set_time_format_rfc2822();
    }

    log_config.set_target_level(log::LevelFilter::Off);
    log_config.set_location_level(log::LevelFilter::Off);
    log_config.add_filter_ignore_str("sled");
    log_config.add_filter_ignore_str("mio");

    let log_level = if args.daemon {
        log::LevelFilter::Info
    } else {
        match args.verbose {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        }
    };

    let color_choice = if args.daemon {
        simplelog::ColorChoice::Never
    } else {
        simplelog::ColorChoice::Auto
    };

    simplelog::TermLogger::init(
        log_level,
        log_config.build(),
        simplelog::TerminalMode::Mixed,
        color_choice,
    )?;

    if args.daemon {
        log::debug!("Running service as daemon");
    }

    log::trace!("{:#?}", config);

    daemonize(&config).await
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Actuator {
    Boom = 0,
    Arm = 4,
    Bucket = 5,
    Slew = 1,
    LimpLeft = 3,
    LimpRight = 2,
}

impl From<Actuator> for u32 {
    fn from(value: Actuator) -> Self {
        value as u32
    }
}

pub enum HydraulicMotion {
    /// Stop all motion until resumed.
    StopAll,
    /// Resume all motion.
    ResumeAll,
    /// Drive straight forward or backwards.
    StraightDrive(i16),
    /// Stop motion on actuators.
    Stop(Vec<Actuator>),
    /// Change motion on actuators.
    Change(Vec<(Actuator, i16)>),
}

#[allow(dead_code)]
impl HydraulicMotion {
    /// Maximum power setting.
    const POWER_MAX: i16 = i16::MAX;
    /// Neutral power setting.
    const POWER_NEUTRAL: i16 = 0;
    /// Minimum power setting.
    const POWER_MIN: i16 = i16::MIN;
}

impl ToMotion for HydraulicMotion {
    fn to_motion(self) -> Motion {
        match self {
            HydraulicMotion::StopAll => glonax::transport::Motion {
                r#type: glonax::transport::motion::MotionType::StopAll.into(),
                changes: vec![],
            },
            HydraulicMotion::ResumeAll => glonax::transport::Motion {
                r#type: glonax::transport::motion::MotionType::ResumeAll.into(),
                changes: vec![],
            },
            HydraulicMotion::StraightDrive(value) => glonax::transport::Motion {
                r#type: glonax::transport::motion::MotionType::Change.into(),
                changes: vec![
                    glonax::transport::motion::ChangeSet {
                        actuator: Actuator::LimpLeft.into(),
                        value: value as i32,
                    },
                    glonax::transport::motion::ChangeSet {
                        actuator: Actuator::LimpRight.into(),
                        value: value as i32,
                    },
                ],
            },
            HydraulicMotion::Stop(v) => glonax::transport::Motion {
                r#type: glonax::transport::motion::MotionType::Change.into(),
                changes: v
                    .iter()
                    .map(|a| glonax::transport::motion::ChangeSet {
                        actuator: (*a).into(),
                        value: 0,
                    })
                    .collect(),
            },
            HydraulicMotion::Change(v) => glonax::transport::Motion {
                r#type: glonax::transport::motion::MotionType::Change.into(),
                changes: v
                    .iter()
                    .map(|(a, va)| glonax::transport::motion::ChangeSet {
                        actuator: (*a).into(),
                        value: *va as i32,
                    })
                    .collect(),
            },
        }
    }
}

struct InputState {
    /// Enable or disable drive lock.
    ///
    /// The drive locks allows two actuators to act at the same
    /// time with a single command.
    drive_lock: bool,
}

impl InputState {
    /// Try to convert input scancode to motion.
    ///
    /// Each individual scancode is mapped to its own motion
    /// structure. This way an input scancode can be more or
    /// less sensitive based on the actuator (and input control).
    fn try_from_input_device(&mut self, input: Scancode) -> Result<HydraulicMotion, ()> {
        match input {
            Scancode::LeftStickX(value) => Ok(HydraulicMotion::Change(vec![(
                Actuator::Slew,
                value.ramp(3072),
            )])),
            Scancode::LeftStickY(value) => Ok(HydraulicMotion::Change(vec![(
                Actuator::Arm,
                value.ramp(3072),
            )])),
            Scancode::RightStickX(value) => Ok(HydraulicMotion::Change(vec![(
                Actuator::Bucket,
                value.ramp(4096),
            )])),
            Scancode::RightStickY(value) => Ok(HydraulicMotion::Change(vec![(
                Actuator::Boom,
                value.ramp(3072),
            )])),
            Scancode::LeftTrigger(value) => {
                if self.drive_lock {
                    Ok(HydraulicMotion::StraightDrive(value.ramp(2048)))
                } else {
                    Ok(HydraulicMotion::Change(vec![(
                        Actuator::LimpLeft,
                        value.ramp(2048),
                    )]))
                }
            }
            Scancode::RightTrigger(value) => {
                if self.drive_lock {
                    Ok(HydraulicMotion::StraightDrive(value.ramp(2048)))
                } else {
                    Ok(HydraulicMotion::Change(vec![(
                        Actuator::LimpRight,
                        value.ramp(2048),
                    )]))
                }
            }
            Scancode::Cancel(ButtonState::Pressed) => Ok(HydraulicMotion::StopAll),
            Scancode::Cancel(ButtonState::Released) => Ok(HydraulicMotion::ResumeAll),
            Scancode::Restrict(ButtonState::Pressed) => {
                self.drive_lock = true;
                Err(())
            }
            Scancode::Restrict(ButtonState::Released) => {
                self.drive_lock = false;
                Ok(HydraulicMotion::StraightDrive(
                    HydraulicMotion::POWER_NEUTRAL,
                ))
            }
            _ => {
                log::warn!("Scancode not mapped to action");
                Err(()) // TODO:
            }
        }
    }
}

async fn daemonize(config: &config::InputConfig) -> anyhow::Result<()> {
    let mut client =
        glonax::transport::vehicle_management_client::VehicleManagementClient::connect(
            config.address.clone(),
        )
        .await?;

    let mut input_device = gamepad::Gamepad::new(std::path::Path::new(&config.device)).await;

    let mut input_state = InputState { drive_lock: false };

    while let Ok(input) = input_device.next().await {
        if let Ok(motion) = input_state.try_from_input_device(input) {
            let motion = motion.to_motion();
            log::debug!("{}", motion);

            client.motion_command(motion).await?;
        }
    }

    Ok(())
}
