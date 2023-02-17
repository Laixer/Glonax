// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};

mod config;
mod gamepad;

const DEVICE_NET_LOCAL_ADDR: u8 = 0x9f;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax input daemon", long_about = None)]
struct Args {
    /// CAN network interface.
    interface: String,

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
        interface: args.interface,
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

async fn daemonize(config: &config::InputConfig) -> anyhow::Result<()> {
    use glonax::core::motion::ToMotion;
    use glonax::device::{Hcu, MotionDevice};
    use glonax::kernel::excavator::Excavator;
    use glonax::net::J1939Network;
    use glonax::Operand;

    let mut runtime = glonax::RuntimeBuilder::<Excavator>::from_config(config)?.build();

    let mut input_device = gamepad::Gamepad::new(std::path::Path::new(&config.device)).await;

    let net = std::sync::Arc::new(J1939Network::new(&config.interface, DEVICE_NET_LOCAL_ADDR)?);

    let mut motion_device = Hcu::new(net);

    while let Ok(input) = input_device.next().await {
        if let Ok(motion) = runtime.operand.try_from_input_device(input) {
            motion_device.actuate(motion.to_motion()).await;
        }
    }

    Ok(())
}
