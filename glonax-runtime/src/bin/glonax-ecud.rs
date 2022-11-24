// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

#[derive(Parser)]
#[command(author = "Copyright (C) 2022 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "ECU Daemon", long_about = None)]
struct Args {
    /// CAN network interface.
    interface: String,

    /// ECU network bind address.
    #[arg(short, long, default_value = "0.0.0.0:54910")]
    address: String,

    /// Disable machine motion (frozen mode).
    #[arg(long)]
    disable_motion: bool,

    /// Run motion requests slow.
    #[arg(long)]
    slow_motion: bool,

    /// Record telemetrics to disk.
    #[arg(long)]
    trace: bool,

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

    let mut config = glonax::EcuConfig {
        address: args.address,
        global: glonax::GlobalConfig::default(),
    };

    config.global.interface = args.interface;
    config.global.enable_motion = !args.disable_motion;
    config.global.enable_trace = args.trace;
    config.global.slow_motion = args.slow_motion;
    config.global.daemon = args.daemon;

    if let Some(workers) = args.workers {
        config.global.runtime_workers = workers;
    }

    let mut log_config = simplelog::ConfigBuilder::new();
    if args.daemon {
        log_config.set_time_level(log::LevelFilter::Off);
        log_config.set_thread_level(log::LevelFilter::Off);
        log_config.set_target_level(log::LevelFilter::Off);
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

    glonax::runtime_ecu(&config)?;

    Ok(())
}