// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{App, Arg};

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() -> anyhow::Result<()> {
    let matches = App::new(BIN_NAME)
        .version(PKG_VERSION)
        .author("Copyright (C) 2022 Laixer Equipment B.V.")
        .about(PKG_DESCRIPTION)
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .value_name("address:port")
                .help("Network address to bind")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("workspace")
                .short("D")
                .long("workspace")
                .value_name("DIR")
                .help("Workspace directory")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("test")
                .short("t")
                .long("test")
                .help("Test configuration and exit"),
        )
        .arg(
            Arg::with_name("no-auto")
                .short("n")
                .long("no-auto")
                .help("Disable autopilot program"),
        )
        .arg(
            Arg::with_name("no-input")
                .long("no-input")
                .help("Disable input controls"),
        )
        .arg(
            Arg::with_name("no-motion")
                .long("no-motion")
                .help("Disable machine motion (frozen mode)"),
        )
        .arg(
            Arg::with_name("systemd")
                .long("systemd")
                .help("Run as systemd service unit"),
        )
        .arg(
            Arg::with_name("trace")
                .long("trace")
                .help("Record telemetrics to disk"),
        )
        .arg(
            Arg::with_name("workers")
                .long("workers")
                .value_name("N")
                .help("Number of runtime workers")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    let local_config = std::env::current_dir().unwrap().join("glonaxd.toml");

    // Try read configuration from global system location first, then from local directory.
    let mut config = glonax::Config::try_from_file(vec![
        "/etc/glonax/glonaxd.toml",
        local_config.to_str().unwrap(),
    ])?;

    if matches.is_present("no-auto") {
        config.enable_autopilot = false;
    }
    if matches.is_present("no-input") {
        config.enable_input = false;
    }
    if matches.is_present("no-motion") {
        config.enable_motion = false;
    }
    if matches.is_present("trace") {
        config.enable_trace = true;
    }
    if matches.is_present("test") {
        config.enable_test = true;
    }
    if matches.is_present("workers") {
        config.runtime_workers = matches.value_of("workers").unwrap().parse().unwrap();
    }
    if matches.is_present("workspace") {
        config.workspace = matches.value_of("workspace").unwrap().parse().unwrap();
    }

    let mut log_config = simplelog::ConfigBuilder::new();
    if matches.is_present("systemd") {
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

    let log_level = if matches.is_present("systemd") {
        log::LevelFilter::Info
    } else {
        match matches.occurrences_of("v") {
            0 => log::LevelFilter::Error,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            3 | _ => log::LevelFilter::Trace,
        }
    };

    let color_choice = if matches.is_present("systemd") {
        simplelog::ColorChoice::Never
    } else {
        simplelog::ColorChoice::Auto
    };

    simplelog::TermLogger::init(
        log_level,
        log_config.build(),
        simplelog::TerminalMode::Mixed,
        color_choice,
    )
    .unwrap();

    log::trace!("{}", config);

    glonax::start_machine(&config)?;

    Ok(())
}
