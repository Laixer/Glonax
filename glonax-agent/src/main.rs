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
    let host = if !config.address.contains(':') {
        config.address.to_owned() + ":30051"
    } else {
        config.address.to_owned()
    };

    log::debug!("Waiting for connection to {}", host);

    let stream = tokio::net::TcpStream::connect(&host).await?;

    log::info!("Connected to {}", host);

    let mut protocol = glonax::transport::Protocol::new(stream);

    let start = glonax::transport::frame::Start::new(config.global.bin_name.clone());
    protocol.write_frame0(start).await?;

    //     if "lat" in self.machine.gnss and "long" in self.machine.gnss:
    //     data["location"] = [self.machine.gnss["lat"], self.machine.gnss["long"]]
    // if "altitude" in self.machine.gnss:
    //     data["altitude"] = self.machine.gnss["altitude"]
    // if "speed" in self.machine.gnss:
    //     data["speed"] = self.machine.gnss["speed"]
    // if "satellites" in self.machine.gnss:
    //     data["satellites"] = self.machine.gnss["satellites"]

    let memory = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let swap = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let cpu_1 = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let cpu_5 = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let cpu_15 = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));
    let uptime = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let memory_clone = memory.clone();
    let swap_clone = swap.clone();
    let cpu_1_clone = cpu_1.clone();
    let cpu_5_clone = cpu_5.clone();
    let cpu_15_clone = cpu_15.clone();
    let uptime_clone = uptime.clone();

    tokio::spawn(async move {
        use std::sync::atomic::Ordering::SeqCst;

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

            let memory = memory_clone.load(SeqCst);
            let swap = swap_clone.load(SeqCst);
            let cpu_1 = cpu_1_clone.load(SeqCst);
            let cpu_5 = cpu_5_clone.load(SeqCst);
            let cpu_15 = cpu_15_clone.load(SeqCst);
            let uptime = uptime_clone.load(SeqCst);

            map.insert("memory", memory.to_string());
            map.insert("swap", swap.to_string());
            map.insert("cpu_1", cpu_1.to_string());
            map.insert("cpu_5", cpu_5.to_string());
            map.insert("cpu_15", cpu_15.to_string());
            if uptime > 0 {
                map.insert("uptime", uptime.to_string());
            }

            let request_url = url.join(&format!("api/v1/{}/probe", instance)).unwrap();

            let response = client.post(request_url).json(&map).send().await.unwrap();

            if response.status() == 200 {
                log::info!("Probe sent successfully");
            } else {
                log::error!("Probe failed");
            }

            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        }
    });

    while let Ok(message) = protocol.read_frame().await {
        if let glonax::transport::Message::Signal(signal) = message {
            if signal.address == 0x9E {
                if signal.function == 382 {
                    if let glonax::core::Metric::Percent(count) = signal.metric {
                        memory.store(count.into(), std::sync::atomic::Ordering::SeqCst);
                    }
                } else if signal.function == 383 {
                    if let glonax::core::Metric::Percent(count) = signal.metric {
                        swap.store(count.into(), std::sync::atomic::Ordering::SeqCst);
                    }
                } else if signal.function == 593 {
                    if let glonax::core::Metric::Percent(count) = signal.metric {
                        cpu_1.store(count.into(), std::sync::atomic::Ordering::SeqCst);
                    }
                } else if signal.function == 594 {
                    if let glonax::core::Metric::Percent(count) = signal.metric {
                        cpu_5.store(count.into(), std::sync::atomic::Ordering::SeqCst);
                    }
                } else if signal.function == 595 {
                    if let glonax::core::Metric::Percent(count) = signal.metric {
                        cpu_15.store(count.into(), std::sync::atomic::Ordering::SeqCst);
                    }
                } else if signal.function == 0x1A5 {
                    if let glonax::core::Metric::Count(count) = signal.metric {
                        uptime.store(count.into(), std::sync::atomic::Ordering::SeqCst);
                    }
                }
                // else {
                //     log::debug!("{:?}", signal);
                // }
            }
        }
    }

    Ok(())
}
