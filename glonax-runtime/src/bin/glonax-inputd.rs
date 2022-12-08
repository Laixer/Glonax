// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};

#[derive(Parser)]
#[command(author = "Copyright (C) 2022 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Input device dispatcher", long_about = None)]
struct Args {
    /// Gamepad input device.
    #[arg(value_hint = ValueHint::FilePath)]
    device: String,

    /// MQTT broker address.
    #[arg(short = 'c', long = "connect", default_value = "127.0.0.1")]
    address: String,

    /// MQTT broker port.
    #[arg(short, long, default_value_t = 1883)]
    port: u16,

    /// MQTT broker username.
    #[arg(short = 'U', long)]
    username: Option<String>,

    /// MQTT broker password.
    #[arg(short = 'P', long)]
    password: Option<String>,

    /// Daemonize the service.
    #[arg(long)]
    daemon: bool,

    /// Number of runtime workers.
    #[arg(long)]
    workers: Option<usize>,

    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut config = glonax::InputConfig {
        device: args.device,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = env!("CARGO_BIN_NAME").to_string();
    config.global.mqtt_host = args.address;
    config.global.mqtt_port = args.port;
    config.global.mqtt_username = args.username;
    config.global.mqtt_password = args.password;
    config.global.daemon = args.daemon;

    if let Some(workers) = args.workers {
        config.global.runtime_workers = workers;
    }

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
            3 | _ => log::LevelFilter::Trace,
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

    glonax::runtime_input(&config)?;

    Ok(())
}
