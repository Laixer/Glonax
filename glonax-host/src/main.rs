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
#[command(about = "Glonax host daemon", long_about = None)]
struct Args {
    /// Refresh interval in milliseconds.
    #[arg(long, default_value_t = 500)]
    interval: u64,
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

    let mut config = config::HostConfig {
        interval: args.interval,
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

async fn daemonize(config: &config::HostConfig) -> anyhow::Result<()> {
    use glonax::channel::SignalSource;

    let mut service = glonax::net::HostService::new();

    loop {
        log::debug!("Waiting for FIFO connection: {}", "signal");

        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .open("signal")
            .await?;

        log::info!("Connected to FIFO: {}", "signal");

        let mut protocol = glonax::transport::Protocol::new(file);

        loop {
            service.refresh();

            let mut signals = vec![];
            service.collect_signals(&mut signals);

            if let Err(e) = protocol.write_all6(signals).await {
                log::error!("Failed to write to socket: {}", e);
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(config.interval)).await;
        }
    }
}