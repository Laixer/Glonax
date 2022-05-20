// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use std::path::PathBuf;

use clap::{Parser, ValueHint};

#[derive(Parser)]
#[clap(author = "Copyright (C) 2022 Laixer Equipment B.V.")]
#[clap(version)]
#[clap(about, long_about = None)]
struct Args {
    /// Workspace directory.
    #[clap(short = 'D', long, value_name = "DIR", value_hint = ValueHint::DirPath)]
    workspace: Option<PathBuf>,

    /// Test configuration and exit.
    #[clap(short, long)]
    test: bool,

    /// Disable autopilot program.
    #[clap(short, long)]
    no_auto: bool,

    /// Disable input controls.
    #[clap(long)]
    no_input: bool,

    /// Disable machine motion (frozen mode).
    #[clap(long)]
    no_motion: bool,

    /// Run as systemd service.
    #[clap(long)]
    systemd: bool,

    /// Record telemetrics to disk.
    #[clap(long)]
    trace: bool,

    /// Number of runtime workers.
    #[clap(long)]
    workers: Option<usize>,

    /// Level of verbosity.
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let local_config = std::env::current_dir()?.join("glonaxd.toml");

    // Try read configuration from global system location first, then from local directory.
    let mut config = glonax::Config::try_from_file(vec![
        "/etc/glonax/glonaxd.toml",
        local_config.to_str().unwrap(),
    ])?;

    config.enable_autopilot = !args.no_auto;
    config.enable_input = !args.no_input;
    config.enable_motion = !args.no_motion;
    config.enable_trace = args.trace;
    config.enable_test = args.test;

    if let Some(workers) = args.workers {
        config.runtime_workers = workers;
    }
    if let Some(workspace) = args.workspace {
        config.workspace = workspace;
    }

    let mut log_config = simplelog::ConfigBuilder::new();
    if args.systemd {
        log_config.set_time_level(log::LevelFilter::Off);
        log_config.set_thread_level(log::LevelFilter::Off);
        log_config.set_target_level(log::LevelFilter::Off);
    } else {
        log_config.set_time_to_local(true);
        log_config.set_time_format("%X %6f".to_owned());
    }

    log_config.set_target_level(log::LevelFilter::Off);
    log_config.set_location_level(log::LevelFilter::Off);
    log_config.add_filter_ignore_str("sled");
    log_config.add_filter_ignore_str("mio");

    let log_level = if args.systemd {
        log::LevelFilter::Info
    } else {
        match args.verbose {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            3 | _ => log::LevelFilter::Trace,
        }
    };

    let color_choice = if args.systemd {
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

    log::trace!("{}", config);

    glonax::start_machine(&config)?;

    Ok(())
}
