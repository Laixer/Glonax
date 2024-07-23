// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueEnum, ValueHint};

mod config;
mod gamepad;
mod input;
mod joystick;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ControlMode {
    /// Xbox controller.
    Xbox,
    /// Logitech joystick (solo mode).
    LogitechSolo,
    /// Logitech joystick (right mode).
    LogitechRight,
    /// Logitech joystick (left mode).
    LogitechLeft,
}

#[derive(Parser)]
#[command(author = "Copyright (C) 2024 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax input daemon", long_about = None)]
struct Args {
    /// Configuration file.
    #[arg(
        short = 'c',
        long = "config",
        alias = "conf",
        default_value = "/etc/glonax.conf",
        value_name = "FILE",
        value_hint = ValueHint::FilePath
    )]
    config: std::path::PathBuf,
    /// Socket path.
    #[arg(
        short = 's',
        long = "socket",
        value_hint = ValueHint::FilePath
    )]
    path: Option<std::path::PathBuf>,
    /// Gamepad input device.
    #[arg(value_hint = ValueHint::FilePath)]
    device: String, // TODO: Why not use pathbuf?
    /// Configure failsafe mode.
    #[arg(short, long, default_value_t = true)]
    fail_safe: bool,
    /// Input commands will translate to the full motion range.
    #[arg(long)]
    full_motion: bool,
    /// Control mode.
    #[arg(short, long)]
    mode: ControlMode,
    /// Quiet output (no logging).
    #[arg(long)]
    quiet: bool,
    /// Daemonize the service.
    #[arg(short = 'D', long)]
    daemon: bool,
    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use log::LevelFilter;

    let args = Args::parse();

    let config: config::Config = glonax::from_file(&args.config)?;

    let is_daemon = args.daemon;
    if is_daemon {
        log::set_max_level(LevelFilter::Debug);
        log::set_boxed_logger(Box::new(glonax::logger::SystemdLogger))?;
    } else {
        let log_level = if args.quiet {
            LevelFilter::Off
        } else {
            match args.verbose {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                _ => LevelFilter::Trace,
            }
        };

        simplelog::TermLogger::init(
            log_level,
            simplelog::ConfigBuilder::new()
                .set_target_level(LevelFilter::Off)
                .set_location_level(LevelFilter::Off)
                .build(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        )?;
    }

    if is_daemon {
        log::info!("Running service as daemon");
    }

    log::trace!("{:#?}", config);

    run(config, args).await
}

async fn run(config: config::Config, args: Args) -> anyhow::Result<()> {
    let bin_name = env!("CARGO_BIN_NAME").to_string();

    let socket_path = args
        .path
        .unwrap_or_else(|| config.unix_listener.path.clone());

    glonax::log_system();

    log::info!("Starting {}", bin_name);
    log::debug!("Runtime version: {}", glonax::consts::VERSION);

    let mut joystick = joystick::Joystick::open(std::path::Path::new(&args.device)).await?;

    log::debug!("Using joystick {}", args.device);

    let mut input_device: Box<dyn crate::gamepad::InputDevice> = match args.mode {
        ControlMode::Xbox => Box::<gamepad::XboxController>::default(),
        ControlMode::LogitechSolo => Box::new(gamepad::LogitechJoystick::solo_mode()),
        ControlMode::LogitechRight => Box::new(gamepad::LogitechJoystick::right_mode()),
        ControlMode::LogitechLeft => Box::new(gamepad::LogitechJoystick::left_mode()),
    };

    let mut input_state = input::InputState {
        drive_lock: false,
        motion_lock: true,
        limit_motion: !args.full_motion,
        engine_rpm: 0,
    };

    if args.fail_safe {
        log::info!("Failsafe mode is enabled");
    }
    if input_state.limit_motion {
        log::info!("Motion range is limited");
    } else {
        log::info!("Full motion range is enabled");
    }
    if input_state.motion_lock {
        log::info!("Motion is locked on startup");
    }

    let user_agent = format!("{}/{}", bin_name, glonax::consts::VERSION);
    let (mut client, instance) = if args.fail_safe {
        glonax::protocol::unix_connect_safe(&socket_path, user_agent).await?
    } else {
        glonax::protocol::unix_connect(&socket_path, user_agent).await?
    };

    log::debug!("Connected to {}", socket_path.display());
    log::info!("{}", instance);

    if !glonax::is_compatibile(instance.version()) {
        return Err(anyhow::anyhow!("Incompatible runtime version"));
    }

    loop {
        let event = joystick.next_event().await?;
        if let Some(code) = input_device.map(&event) {
            if let Some(object) = input_state.try_from(code) {
                log::trace!("{:?}", object);

                match object {
                    glonax::core::Object::Motion(motion) => client.send_packet(&motion).await?,
                    glonax::core::Object::Engine(engine) => client.send_packet(&engine).await?,
                    _ => {}
                }
            }
        }
    }
}
