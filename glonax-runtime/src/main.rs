// Copyright (C) 2021 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{App, Arg};

const SERIAL_HYDRAULIC: &str = "/dev/ttyUSB0";

#[tokio::main]
async fn main() {
    let matches = App::new("Glonax daemon")
        .version("0.3.1")
        .author("Copyright (C) 2021 Laixer Equipment B.V.")
        .about("Heavy machinery controller daemon")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Configuration file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("listen")
                .short("l")
                .value_name("address:port")
                .help("Network address to bind")
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
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    let mut config = glonax::Config {
        motion_device: SERIAL_HYDRAULIC.to_owned(),
        ..Default::default()
    };

    if matches.is_present("no-auto") {
        config.enable_autopilot = false;
    }
    if matches.is_present("no-input") {
        config.enable_command = false;
    }

    let log_config = simplelog::ConfigBuilder::new()
        .set_time_to_local(true)
        .set_time_format("%X %6f".to_owned())
        .set_target_level(log::LevelFilter::Trace)
        .add_filter_ignore_str("sled")
        .add_filter_ignore_str("gilrs")
        .build();

    let log_level = match matches.occurrences_of("v") {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 | _ => log::LevelFilter::Trace,
    };

    simplelog::TermLogger::init(
        log_level,
        log_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    let result = glonax::ExcavatorService::from_config(&config)
        .launch()
        .await;

    if let Err(e) = result {
        log::error!("Error: {}", e)
    }
}
