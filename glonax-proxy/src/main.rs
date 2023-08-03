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
#[command(about = "Glonax proxy daemon", long_about = None)]
struct Args {
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

    let mut config = config::ProxyConfig {
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

struct SignalFifo {
    file: std::fs::File,
}

impl SignalFifo {
    pub fn new() -> anyhow::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("signal")?;

        Ok(Self { file })
    }

    // TODO: read more than one signal at a time
    pub fn fetch(&mut self) -> anyhow::Result<glonax::core::Signal> {
        use std::io::Read;

        let mut buffer = [0u8; std::mem::size_of::<glonax::core::Signal>()];

        self.file.read_exact(&mut buffer)?;

        let signal = glonax::core::Signal::from(&buffer);

        log::trace!("Read signal from channel: {}", signal);

        Ok(signal)
    }
}

impl glonax::channel::SignalChannel for SignalFifo {
    fn push(&mut self, signal: glonax::core::Signal) {
        use std::io::Write;

        log::trace!("Write signal to channel: {}", signal);

        self.file.write(signal.bytes()).unwrap();
    }
}

async fn daemonize(config: &config::ProxyConfig) -> anyhow::Result<()> {
    let mut channel = SignalFifo::new()?;

    log::debug!("Starting host services");

    while let Ok(signal) = channel.fetch() {
        log::info!("Received signal: {}", signal);
    }

    log::debug!("{} was shutdown gracefully", config.global.bin_name);

    Ok(())
}
