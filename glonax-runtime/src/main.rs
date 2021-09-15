// Copyright (C) 2021 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{App, Arg};

fn main() {
    let matches = App::new("Glonax daemon")
        .version("0.3.1")
        .author("Copyright (C) 2021 Laixer Equipment B.V.")
        .about("Heavy machinery controller daemon")
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
            Arg::with_name("record")
                .short("R")
                .long("record")
                .help("Record log to disk"),
        )
        .arg(
            Arg::with_name("no-auto")
                .long("no-auto")
                .help("Disable autopilot program"),
        )
        .arg(
            Arg::with_name("no-input")
                .long("no-input")
                .help("Disable input controls"),
        )
        .arg(
            Arg::with_name("systemd")
                .long("systemd")
                .help("Run as systemd service unit"),
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

    let mut config = glonax::Config {
        motion_device: "/dev/ttyUSB1".to_owned(),
        metric_devices: vec!["/dev/ttyUSB0".to_owned()],
        ..Default::default()
    };

    if matches.is_present("no-auto") {
        config.enable_autopilot = false;
    }
    if matches.is_present("no-input") {
        config.enable_command = false;
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

    log_config.set_target_level(log::LevelFilter::Trace);
    log_config.set_location_level(log::LevelFilter::Off);
    log_config.add_filter_ignore_str("sled");
    log_config.add_filter_ignore_str("gilrs");
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

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(8)
        .enable_all()
        .thread_name("glonax-runtime-worker")
        .thread_stack_size(8 * 1024 * 1024)
        .build()
        .unwrap();

    let result = runtime.block_on(async {
        glonax::ExcavatorService::from_config(&config)?
            .launch()
            .await
    });

    if let Err(e) = result {
        log::error!("{}", e)
    }
}
