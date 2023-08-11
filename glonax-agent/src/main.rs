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

    let host = if !config.address.contains(':') {
        config.address.to_owned() + ":30051"
    } else {
        config.address.to_owned()
    };

    struct Telemetry {
        location: Option<glonax::core::Metric>,
        altitude: Option<glonax::core::Metric>,
        speed: Option<glonax::core::Metric>,
        satellites: Option<glonax::core::Metric>,
        memory: Option<glonax::core::Metric>,
        swap: Option<glonax::core::Metric>,
        cpu_1: Option<glonax::core::Metric>,
        cpu_5: Option<glonax::core::Metric>,
        cpu_15: Option<glonax::core::Metric>,
        uptime: Option<glonax::core::Metric>,
    }

    let runtime = glonax::RuntimeBuilder::from_config(config)?
        .with_shutdown()
        .build();

    let telemetrics = Arc::new(RwLock::new(Telemetry {
        location: None,
        altitude: None,
        speed: None,
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

                if let Some(location) = telemetric_lock.location {
                    if let glonax::core::Metric::Coordinates((lat, long)) = location {
                        // map.insert("memory", count.to_string());
                        log::debug!("{}", location);
                    }
                }

                if let Some(altitude) = telemetric_lock.altitude {
                    if let glonax::core::Metric::Altitude(value) = altitude {
                        map.insert("altitude", value.to_string());
                        log::debug!("{}", altitude);
                    }
                }

                if let Some(speed) = telemetric_lock.speed {
                    if let glonax::core::Metric::Speed(value) = speed {
                        map.insert("speed", value.to_string());
                        log::debug!("{}", speed);
                    }
                }

                if let Some(satellites) = telemetric_lock.satellites {
                    if let glonax::core::Metric::Count(value) = satellites {
                        map.insert("satellites", value.to_string());
                        log::debug!("satellites: {}", value);
                    }
                }

                if let Some(memory) = telemetric_lock.memory {
                    if let glonax::core::Metric::Percent(value) = memory {
                        map.insert("memory", value.to_string());
                        log::debug!("memory: {}", memory);
                    }
                }

                if let Some(swap) = telemetric_lock.swap {
                    if let glonax::core::Metric::Percent(count) = swap {
                        map.insert("swap", count.to_string());
                        log::debug!("swap: {}", count);
                    }
                }

                if let Some(cpu_1) = telemetric_lock.cpu_1 {
                    if let glonax::core::Metric::Percent(count) = cpu_1 {
                        map.insert("cpu_1", count.to_string());
                        log::debug!("cpu_1: {}", count);
                    }
                }

                if let Some(cpu_5) = telemetric_lock.cpu_5 {
                    if let glonax::core::Metric::Percent(count) = cpu_5 {
                        map.insert("cpu_5", count.to_string());
                        log::debug!("cpu_5: {}", count);
                    }
                }

                if let Some(cpu_15) = telemetric_lock.cpu_15 {
                    if let glonax::core::Metric::Percent(count) = cpu_15 {
                        map.insert("cpu_15", count.to_string());
                        log::debug!("cpu_15: {}", count);
                    }
                }

                if let Some(uptime) = telemetric_lock.uptime {
                    if let glonax::core::Metric::Count(count) = uptime {
                        map.insert("uptime", count.to_string());
                        log::debug!("uptime: {}", count);
                    }
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

    log::debug!("Waiting for connection to {}", host);

    let mut client = glonax::transport::Client::connect(&host, &config.global.bin_name).await?;

    log::info!("Connected to {}", host);

    let shutdown = runtime.shutdown_signal();

    while let Ok(signal) = client.recv_signal().await {
        if !shutdown.is_empty() {
            break;
        }

        if signal.address == 0x9E {
            if signal.function == 0x17E {
                telemetrics.write().await.memory = Some(signal.metric);
            } else if signal.function == 0x17F {
                telemetrics.write().await.swap = Some(signal.metric);
            } else if signal.function == 0x251 {
                telemetrics.write().await.cpu_1 = Some(signal.metric);
            } else if signal.function == 0x252 {
                telemetrics.write().await.cpu_5 = Some(signal.metric);
            } else if signal.function == 0x253 {
                telemetrics.write().await.cpu_15 = Some(signal.metric);
            } else if signal.function == 0x1A5 {
                telemetrics.write().await.uptime = Some(signal.metric);
            }
        } else if signal.address == 0x01 {
            if signal.function == 0x0 {
                telemetrics.write().await.location = Some(signal.metric);
            } else if signal.function == 0x1 {
                telemetrics.write().await.altitude = Some(signal.metric);
            } else if signal.function == 0x2 {
                telemetrics.write().await.speed = Some(signal.metric);
            } else if signal.function == 0xA {
                telemetrics.write().await.satellites = Some(signal.metric);
            }
        }
    }

    client.shutdown().await?;

    log::debug!("{} was shutdown gracefully", config.global.bin_name);

    Ok(())
}
