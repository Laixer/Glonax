// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};
use glonax::transport::ToMotion;

mod config;
mod gamepad;
mod input;
mod motion;

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
    /// Input commands will translate to the full motion range.
    #[arg(long)]
    full_motion: bool,
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
        full_motion: args.full_motion,
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

async fn daemonize(config: &config::InputConfig) -> anyhow::Result<()> {
    let mut client =
        glonax::transport::vehicle_management_client::VehicleManagementClient::connect(
            config.address.clone(),
        )
        .await?;

    let mut input_device = gamepad::Gamepad::new(std::path::Path::new(&config.device)).await;

    let mut input_state = input::InputState {
        drive_lock: false,
        motion_lock: true,
        limit_motion: !config.full_motion,
    };

    while let Ok(input) = input_device.next().await {
        if let Some(motion) = input_state.try_from(input) {
            let motion = motion.to_motion();
            log::debug!("{}", motion);

            client.motion_command(motion).await?;
        }
    }

    Ok(())
}
