// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

#[derive(Parser)]
#[clap(author = "Copyright (C) 2022 Laixer Equipment B.V.")]
#[clap(version)]
#[clap(about, long_about = None)]
struct Args {
    /// CAN network interface.
    interface: String,

    /// Execute provided program id.
    #[clap(short, long)]
    program: Option<i32>,

    /// Disable autopilot program.
    #[clap(short, long)]
    no_auto: bool,

    /// Disable machine motion (frozen mode).
    #[clap(long)]
    disable_motion: bool,

    /// Run motion requests slow.
    #[clap(long)]
    slow_motion: bool,

    /// Record telemetrics to disk.
    #[clap(long)]
    trace: bool,

    /// Daemonize the service.
    #[clap(long)]
    daemon: bool,

    /// Number of runtime workers.
    #[clap(long)]
    workers: Option<usize>,

    /// Level of verbosity.
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // let local_config = std::env::current_dir()?.join("glonaxd.toml");

    // Try read configuration from global system location first, then from local directory.
    // let mut config = glonax::Config::try_from_file(vec![
    //     "/etc/glonax/glonaxd.toml",
    //     local_config.to_str().unwrap(),
    // ])?;
    let mut config = glonax::ProgramConfig {
        enable_autopilot: !args.no_auto,
        program_id: args.program,
        ..Default::default()
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

    glonax::runtime_program(&config)?;

    Ok(())
}
