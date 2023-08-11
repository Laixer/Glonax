// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

use std::collections::HashMap;

const BASE_URL: &str = "https://cymbion-oybqn.ondigitalocean.app/";
const VERSION: &str = "102";

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax agent", long_about = None)]
struct Args {
    /// Remote network address.
    #[arg(short = 'c', long = "connect", default_value = "127.0.0.1:30051")]
    address: String,
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

    let mut config = config::AgentConfig {
        address: args.address,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = env!("CARGO_BIN_NAME").to_string();
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

async fn daemonize(config: &config::AgentConfig) -> anyhow::Result<()> {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    struct Telemetry {
        location: Option<(f32, f32)>,
        altitude: Option<f32>,
        speed: Option<f32>,
        heading: Option<f32>,
        satellites: Option<u64>,
        memory: Option<i32>,
        swap: Option<i32>,
        cpu_1: Option<i32>,
        cpu_5: Option<i32>,
        cpu_15: Option<i32>,
        uptime: Option<u64>,
    }

    let runtime = glonax::RuntimeBuilder::from_config(config)?
        .with_shutdown()
        .build();

    let telemetrics = Arc::new(RwLock::new(Telemetry {
        location: None,
        altitude: None,
        speed: None,
        heading: None,
        satellites: None,
        memory: None,
        swap: None,
        cpu_1: None,
        cpu_5: None,
        cpu_15: None,
        uptime: None,
    }));

    let telemetrics_clone = telemetrics.clone();

    tokio::spawn(async move {
        log::debug!("Starting host service");

        let url = reqwest::Url::parse(BASE_URL).unwrap();

        let instance = "7e796adf-c4e5-40e2-88d8-d79d5db61e95";

        let client = reqwest::Client::builder()
            .user_agent("glonax-agent/0.1.0")
            .timeout(std::time::Duration::from_secs(5))
            .https_only(true)
            .build()
            .unwrap();

        loop {
            let mut map = HashMap::new();
            map.insert("version", VERSION.to_string());
            map.insert("status", "HEALTHY".to_string());
            map.insert("name", "glonax-agent".to_string());

            {
                let telemetric_lock = telemetrics_clone.read().await;

                if let Some((lat, long)) = telemetric_lock.location {
                    log::trace!("{} {}", lat, long);
                }

                if let Some(altitude) = telemetric_lock.altitude {
                    map.insert("altitude", altitude.to_string());
                    log::trace!("{}", altitude);
                }

                if let Some(speed) = telemetric_lock.speed {
                    map.insert("speed", speed.to_string());
                    log::trace!("{}", speed);
                }

                if let Some(heading) = telemetric_lock.heading {
                    map.insert("heading", heading.to_string());
                    log::trace!("{}", heading);
                }

                if let Some(satellites) = telemetric_lock.satellites {
                    map.insert("satellites", satellites.to_string());
                    log::trace!("satellites: {}", satellites);
                }

                if let Some(memory) = telemetric_lock.memory {
                    map.insert("memory", memory.to_string());
                    log::trace!("memory: {}", memory);
                }

                if let Some(swap) = telemetric_lock.swap {
                    map.insert("swap", swap.to_string());
                    log::trace!("swap: {}", swap);
                }

                if let Some(cpu_1) = telemetric_lock.cpu_1 {
                    map.insert("cpu_1", cpu_1.to_string());
                    log::trace!("cpu_1: {}", cpu_1);
                }

                if let Some(cpu_5) = telemetric_lock.cpu_5 {
                    map.insert("cpu_5", cpu_5.to_string());
                    log::trace!("cpu_5: {}", cpu_5);
                }

                if let Some(cpu_15) = telemetric_lock.cpu_15 {
                    map.insert("cpu_15", cpu_15.to_string());
                    log::trace!("cpu_15: {}", cpu_15);
                }

                if let Some(uptime) = telemetric_lock.uptime {
                    map.insert("uptime", uptime.to_string());
                    log::trace!("uptime: {}", uptime);
                }
            }

            let request_url = url.join(&format!("api/v1/{}/probe", instance)).unwrap();

            let response = client.post(request_url).json(&map).send().await.unwrap();

            if response.status() == 200 {
                log::info!("Probe sent successfully");
            } else {
                log::error!("Probe failed");
            }

            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    log::debug!("Waiting for connection to {}", config.address);

    let mut client =
        glonax::transport::Client::connect(&config.address, &config.global.bin_name).await?;

    log::info!("Connected to {}", config.address);

    let shutdown = runtime.shutdown_signal();

    while let Ok(signal) = client.recv_signal().await {
        if !shutdown.is_empty() {
            break;
        }

        if signal.address == 0x9E {
            let mut telemetric_lock = telemetrics.write().await;

            match signal.into() {
                glonax::net::HostMessage::Memory(memory) => {
                    telemetric_lock.memory = Some(memory);
                }
                glonax::net::HostMessage::Swap(swap) => {
                    telemetric_lock.swap = Some(swap);
                }
                glonax::net::HostMessage::Cpu1(cpu_1) => {
                    telemetric_lock.cpu_1 = Some(cpu_1);
                }
                glonax::net::HostMessage::Cpu5(cpu_5) => {
                    telemetric_lock.cpu_5 = Some(cpu_5);
                }
                glonax::net::HostMessage::Cpu15(cpu_15) => {
                    telemetric_lock.cpu_15 = Some(cpu_15);
                }
                glonax::net::HostMessage::Timestamp(_timestamp) => {
                    // telemetric_lock.uptime = Some(timestamp);
                }
                glonax::net::HostMessage::Uptime(uptime) => {
                    telemetric_lock.uptime = Some(uptime);
                }
            }
        } else if signal.address == 0x01 {
            let mut telemetric_lock = telemetrics.write().await;

            match signal.into() {
                glonax::net::NMEAMessage2::Coordinates(coordinates) => {
                    telemetric_lock.location = Some(coordinates);
                }
                glonax::net::NMEAMessage2::Altitude(altitude) => {
                    telemetric_lock.altitude = Some(altitude);
                }
                glonax::net::NMEAMessage2::Speed(speed) => {
                    telemetric_lock.speed = Some(speed);
                }
                glonax::net::NMEAMessage2::Heading(heading) => {
                    telemetric_lock.heading = Some(heading);
                }
                glonax::net::NMEAMessage2::Satellites(satellites) => {
                    telemetric_lock.satellites = Some(satellites);
                }
            }
        }
    }

    client.shutdown().await?;

    log::debug!("{} was shutdown gracefully", config.global.bin_name);

    Ok(())
}
