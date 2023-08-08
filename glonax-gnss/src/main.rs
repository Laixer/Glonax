// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax GNSS daemon", long_about = None)]
struct Args {
    /// Serial device.
    device: std::path::PathBuf,
    /// Serial baud rate.
    #[arg(long, default_value_t = 9600)]
    baud_rate: usize,
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

    let bin_name = env!("CARGO_BIN_NAME");

    let mut config = config::GnssConfig {
        device: args.device,
        baud_rate: args.baud_rate,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = bin_name.to_string();
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

async fn daemonize(config: &config::GnssConfig) -> anyhow::Result<()> {
    use glonax::channel::SignalSource;
    use tokio::io::{AsyncBufReadExt, BufReader};

    log::info!("Starting GNSS service");

    let serial = glonax_serial::Uart::open(
        &config.device,
        glonax_serial::BaudRate::from_speed(config.baud_rate),
    )?;

    let reader = BufReader::new(serial);
    let mut lines = reader.lines();

    let service = glonax::net::NMEAService;

    loop {
        log::debug!("Waiting for FIFO connection: {}", "signal");

        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .open("signal")
            .await?;

        log::debug!("Connected to FIFO: {}", "signal");

        let mut protocol = glonax::transport::Protocol::new(file);

        while let Some(line) = lines.next_line().await? {
            if let Some(message) = service.decode(line) {
                let mut signals = vec![];
                message.collect_signals(&mut signals);

                if let Err(e) = protocol.write_all6(signals).await {
                    log::error!("Failed to write to socket: {}", e);
                    break;
                }
            }
        }
    }
}
