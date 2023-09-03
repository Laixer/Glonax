// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

const HOST: &str = "https://cymbion-oybqn.ondigitalocean.app";
const VERSION: &str = "102";

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax agent", long_about = None)]
struct Args {
    /// Probe interval in seconds.
    #[arg(short, long, default_value_t = 60)]
    interval: u64,
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

    let (instance, _) = glonax::channel::recv_instance().await?;

    let mut config = config::AgentConfig {
        interval: args.interval,
        instance,
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

    daemonize(&mut config).await
}

async fn daemonize(config: &config::AgentConfig) -> anyhow::Result<()> {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[derive(Debug, Clone, serde_derive::Serialize)]
    struct Telemetry {
        version: String,
        status: String,
        name: String,
        location: Option<(f32, f32)>,
        altitude: Option<f32>,
        speed: Option<f32>,
        heading: Option<f32>,
        satellites: Option<u8>,
        memory: Option<u64>,
        swap: Option<u64>,
        cpu_1: Option<f64>,
        cpu_5: Option<f64>,
        cpu_15: Option<f64>,
        uptime: Option<u64>,
    }

    let telemetrics = Arc::new(RwLock::new(Telemetry {
        version: VERSION.to_string(),
        status: "HEALTHY".to_string(),
        name: config.instance.name.clone(),
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
    let instance_clone = config.instance.clone();

    let interval = config.interval;

    tokio::spawn(async move {
        log::debug!("Starting host service");

        let url = reqwest::Url::parse(HOST).unwrap();

        let client = reqwest::Client::builder()
            .user_agent("glonax-agent/0.1.0")
            .timeout(std::time::Duration::from_secs(5))
            .https_only(true)
            .build()
            .unwrap();

        let request_url = url
            .join(&format!("api/v1/{}/probe", instance_clone.id))
            .unwrap();

        loop {
            let data = { telemetrics_clone.read().await.clone() };

            let response = client
                .post(request_url.clone())
                .json(&data)
                .send()
                .await
                .unwrap();

            if response.status() == 200 {
                log::info!("Probe sent successfully");
            } else {
                log::error!("Probe failed, status: {}", response.status());
            }

            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
    });

    let socket = glonax::channel::broadcast_bind().await?;

    let mut buffer = [0u8; 1024];

    log::debug!("Listening for signals");

    loop {
        let (size, _) = socket.recv_from(&mut buffer).await?;

        if let Ok(frame) = glonax::transport::frame::Frame::try_from(&buffer[..size]) {
            if frame.message == glonax::transport::frame::FrameMessage::Signal {
                let signal =
                    glonax::core::Signal::try_from(&buffer[frame.payload_range()]).unwrap();

                match signal.metric {
                    glonax::core::Metric::VmsUptime(uptime) => {
                        telemetrics.write().await.uptime = Some(uptime);
                    }
                    glonax::core::Metric::VmsMemoryUsage((memory_used, memory_total)) => {
                        let memory_usage = (memory_used as f64 / memory_total as f64) * 100.0;

                        telemetrics.write().await.memory = Some(memory_usage as u64);
                    }
                    glonax::core::Metric::VmsSwapUsage(swap) => {
                        telemetrics.write().await.swap = Some(swap);
                    }
                    glonax::core::Metric::VmsCpuLoad(cpu_load) => {
                        telemetrics.write().await.cpu_1 = Some(cpu_load.0);
                        telemetrics.write().await.cpu_5 = Some(cpu_load.1);
                        telemetrics.write().await.cpu_15 = Some(cpu_load.2);
                    }
                    glonax::core::Metric::GnssLatLong(lat_long) => {
                        telemetrics.write().await.location = Some(lat_long);
                    }
                    glonax::core::Metric::GnssAltitude(altitude) => {
                        telemetrics.write().await.altitude = Some(altitude);
                    }
                    glonax::core::Metric::GnssSpeed(speed) => {
                        telemetrics.write().await.speed = Some(speed);
                    }
                    glonax::core::Metric::GnssHeading(heading) => {
                        telemetrics.write().await.heading = Some(heading);
                    }
                    glonax::core::Metric::GnssSatellites(satellites) => {
                        telemetrics.write().await.satellites = Some(satellites);
                    }
                    _ => {}
                }
            }
        }
    }
}
