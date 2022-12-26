// Copyright (C) 2022 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::{Parser, ValueHint};

#[derive(Parser)]
#[command(author = "Copyright (C) 2022 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax client interface", long_about = None)]
struct Args {
    /// Program instruction file.
    #[arg(value_hint = ValueHint::FilePath)]
    file: String,

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

    /// Level of verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut config = glonax::ClientConfig {
        file: args.file,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = env!("CARGO_BIN_NAME").to_string();
    config.global.mqtt_host = args.address;
    config.global.mqtt_port = args.port;
    config.global.mqtt_username = args.username;
    config.global.mqtt_password = args.password;

    let log_config = simplelog::ConfigBuilder::new()
        .set_time_level(log::LevelFilter::Off)
        .set_thread_level(log::LevelFilter::Off)
        .set_target_level(log::LevelFilter::Off)
        .set_location_level(log::LevelFilter::Off)
        .add_filter_ignore_str("sled")
        .add_filter_ignore_str("mio")
        .build();

    let log_level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    simplelog::TermLogger::init(
        log_level,
        log_config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )?;

    log::trace!("{:#?}", config);

    glonax::runtime_client(&config)?;

    Ok(())
}
