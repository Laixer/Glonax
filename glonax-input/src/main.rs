// Copyright (C) 2024 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueEnum, ValueHint};

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
    /// Remote network address.
    #[arg(short = 'c', long = "connect", default_value = "127.0.0.1")]
    address: String,
    /// Gamepad input device.
    #[arg(value_hint = ValueHint::FilePath)]
    device: String,
    /// Configure failsafe mode.
    #[arg(short, long)]
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

    let is_daemon = std::env::var("INVOCATION_ID").is_ok() || args.daemon;

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

    run(args).await
}

async fn run(args: Args) -> anyhow::Result<()> {
    let bin_name = env!("CARGO_BIN_NAME").to_string();

    let mut address = args.address.clone();

    if !address.contains(':') {
        address.push(':');
        address.push_str(&glonax::consts::DEFAULT_NETWORK_PORT.to_string());
    }

    let address = std::net::ToSocketAddrs::to_socket_addrs(&address)?
        .next()
        .unwrap();

    let mut joystick = joystick::Joystick::open(std::path::Path::new(&args.device)).await?;

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
    };

    if input_state.limit_motion {
        log::info!("Motion range is limited");
    }
    if input_state.motion_lock {
        log::info!("Motion is locked on startup");
    }

    glonax::log_system();

    log::info!("Starting {}", bin_name);
    log::debug!("Runtime version: {}", glonax::consts::VERSION);
    log::debug!("Waiting for connection to {}", address);

    let (mut client, instance) = glonax::protocol::client::connect_with(
        address.to_owned(),
        format!("{}/{}", bin_name, glonax::consts::VERSION),
        true,
        args.fail_safe,
    )
    .await?;

    log::info!("Connected to {}", address);

    log::info!("{}", instance);

    loop {
        let event = joystick.next_event().await?;
        if let Some(code) = input_device.map(&event) {
            if let Some(motion) = input_state.try_from(code) {
                log::trace!("{}", motion);

                client.send_packet(&motion).await?
            }
        }
    }
}
