// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};

mod config;
mod gamepad;
mod input;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
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

    let mut address = args.address.clone();

    if !address.contains(':') {
        address.push(':');
        address.push_str(&glonax::consts::DEFAULT_NETWORK_PORT.to_string());
    }

    let address = std::net::ToSocketAddrs::to_socket_addrs(&address)?
        .next()
        .unwrap();

    let mut config = config::InputConfig {
        address,
        device: args.device,
        fail_safe: args.fail_safe,
        full_motion: args.full_motion,
        ..Default::default()
    };

    config.global.bin_name = env!("CARGO_BIN_NAME").to_string();
    config.global.daemon = args.daemon;

    let mut log_config = simplelog::ConfigBuilder::new();
    if args.daemon {
        log_config.set_time_level(log::LevelFilter::Off);
        log_config.set_thread_level(log::LevelFilter::Off);
    }

    log_config.set_target_level(log::LevelFilter::Off);
    log_config.set_location_level(log::LevelFilter::Off);
    log_config.add_filter_ignore_str("sled");
    log_config.add_filter_ignore_str("mio");

    let log_level = if args.daemon {
        LevelFilter::Info
    } else if args.quiet {
        LevelFilter::Off
    } else {
        match args.verbose {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
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

    let mut input_device = gamepad::Gamepad::new(std::path::Path::new(&config.device)).await?;

    let mut input_state = input::InputState {
        drive_lock: false,
        motion_lock: true,
        limit_motion: !config.full_motion,
    };

    if input_state.limit_motion {
        log::info!("Motion range is limited");
    }
    if input_state.motion_lock {
        log::info!("Motion is locked on startup");
    }

    log::debug!("Waiting for connection to {}", config.address);

    let mut client = glonax::transport::ConnectionOptions::new()
        .read(false)
        .control(true)
        .failsafe(config.fail_safe)
        .connect(
            config.address.to_owned(),
            format!("{}/{}", config.global.bin_name, glonax::consts::VERSION),
        )
        .await?;

    client
        .send_request(glonax::transport::frame::FrameMessage::Instance)
        .await?;

    let frame = client.read_frame().await?;
    match frame.message {
        glonax::transport::frame::FrameMessage::Instance => {
            let instance = client
                .packet::<glonax::core::Instance>(frame.payload_length)
                .await?;

            log::info!("Instance ID: {}", instance.id);
            log::info!("Instance Model: {}", instance.model);
            log::info!("Instance Name: {}", instance.name);
        }
        _ => {
            log::error!("Invalid response from server");
            return Ok(());
        }
    }

    log::info!("Connected to {}", config.address);

    while let Ok(input) = input_device.next().await {
        if let Some(motion) = input_state.try_from(input) {
            log::trace!("{}", motion);

            if let Err(e) = client.send_packet(&motion).await {
                log::error!("Failed to write to socket: {}", e);
                break;
            }
        }
    }

    Ok(())
}
