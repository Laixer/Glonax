// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

const HOST: &str = "https://cymbion-oybqn.ondigitalocean.app";

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax agent", long_about = None)]
struct Args {
    /// Probe interval in seconds.
    #[arg(short, long, default_value_t = 60)]
    interval: u64,
    /// Send a probe to remote host.
    #[arg(long)]
    no_probe: bool,
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
        probe: !args.no_probe,
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
    rpm: Option<u16>,
}

impl std::fmt::Display for Telemetry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut status = self.status.clone();

        if let Some(uptime) = self.uptime {
            status.push_str(&format!(" Uptime: {}s", uptime));
        }

        if let Some(memory) = self.memory {
            status.push_str(&format!(" Memory: {}%", memory));
        }

        if let Some(swap) = self.swap {
            status.push_str(&format!(" Swap: {}%", swap));
        }

        if let Some(cpu_1) = self.cpu_1 {
            status.push_str(&format!(" CPU 1: {}%", cpu_1));
        }

        if let Some(cpu_5) = self.cpu_5 {
            status.push_str(&format!(" CPU 5: {}%", cpu_5));
        }

        if let Some(cpu_15) = self.cpu_15 {
            status.push_str(&format!(" CPU 15: {}%", cpu_15));
        }

        if let Some((value_lat, value_long)) = self.location {
            status.push_str(&format!(" Location: ({:.5}, {:.5})", value_lat, value_long));
        }

        if let Some(altitude) = self.altitude {
            status.push_str(&format!(" Altitude: {:.1}m", altitude));
        }

        if let Some(speed) = self.speed {
            status.push_str(&format!(" Speed: {:.1}m/s", speed));
        }

        if let Some(heading) = self.heading {
            status.push_str(&format!(" Heading: {:.1}°", heading));
        }

        if let Some(satellites) = self.satellites {
            status.push_str(&format!(" Satellites: {}", satellites));
        }

        if let Some(rpm) = self.rpm {
            status.push_str(&format!(" RPM: {}", rpm));
        }

        write!(f, "{}", status)
    }
}

async fn daemonize(config: &config::AgentConfig) -> anyhow::Result<()> {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let telemetrics = Arc::new(RwLock::new(Telemetry {
        version: format!(
            "{}{}{}",
            glonax::constants::VERSION_MAJOR,
            glonax::constants::VERSION_MINOR,
            glonax::constants::VERSION_PATCH
        ),
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
        rpm: None,
    }));

    let telemetrics_clone = telemetrics.clone();

    tokio::spawn(async move {
        use glonax::core::{Metric, Signal};
        use glonax::transport::frame::{Frame, FrameMessage};

        let socket = glonax::channel::broadcast_bind()
            .await
            .expect("Failed to bind to socket");

        let mut buffer = [0u8; 1024];

        log::debug!("Listening for signals");

        loop {
            let (size, _) = socket.recv_from(&mut buffer).await.unwrap();

            if let Ok(frame) = Frame::try_from(&buffer[..size]) {
                if frame.message == FrameMessage::Signal {
                    let signal = Signal::try_from(&buffer[frame.payload_range()]).unwrap();

                    match signal.metric {
                        Metric::VmsUptime(uptime) => {
                            telemetrics.write().await.uptime = Some(uptime);
                        }
                        Metric::VmsMemoryUsage((memory_used, memory_total)) => {
                            let memory_usage = (memory_used as f64 / memory_total as f64) * 100.0;

                            telemetrics.write().await.memory = Some(memory_usage as u64);
                        }
                        Metric::VmsSwapUsage((swap_used, swap_total)) => {
                            let swap_usage = (swap_used as f64 / swap_total as f64) * 100.0;

                            telemetrics.write().await.swap = Some(swap_usage as u64);
                        }
                        Metric::VmsCpuLoad((cpu_load_1, cpu_load_5, cpu_load_15)) => {
                            telemetrics.write().await.cpu_1 = Some(cpu_load_1);
                            telemetrics.write().await.cpu_5 = Some(cpu_load_5);
                            telemetrics.write().await.cpu_15 = Some(cpu_load_15);
                        }
                        Metric::GnssLatLong(lat_long) => {
                            telemetrics.write().await.location = Some(lat_long);
                        }
                        Metric::GnssAltitude(altitude) => {
                            telemetrics.write().await.altitude = Some(altitude);
                        }
                        Metric::GnssSpeed(speed) => {
                            telemetrics.write().await.speed = Some(speed);
                        }
                        Metric::GnssHeading(heading) => {
                            telemetrics.write().await.heading = Some(heading);
                        }
                        Metric::GnssSatellites(satellites) => {
                            telemetrics.write().await.satellites = Some(satellites);
                        }
                        Metric::EngineRpm(rpm) => {
                            telemetrics.write().await.rpm = Some(rpm);
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    log::debug!("Starting host service");

    let url = reqwest::Url::parse(HOST).unwrap();

    let client = reqwest::Client::builder()
        .user_agent("glonax-agent/0.1.0")
        .timeout(std::time::Duration::from_secs(5))
        .https_only(true)
        .build()
        .unwrap();

    let request_url = url
        .join(&format!("api/v1/{}/probe", config.instance.id))
        .unwrap();

    loop {
        if config.probe {
            let data = telemetrics_clone.read().await;

            let response = client
                .post(request_url.clone())
                .json(&*data)
                .send()
                .await
                .unwrap();

            if response.status() == 200 {
                log::info!("Probe sent successfully");
            } else {
                log::error!("Probe failed, status: {}", response.status());
            }
        };

        log::trace!("{}", telemetrics_clone.read().await);

        tokio::time::sleep(std::time::Duration::from_secs(config.interval)).await;
    }
}
