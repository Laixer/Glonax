// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

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

    let mut config = config::AgentConfig {
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

async fn daemonize(_config: &config::AgentConfig) -> anyhow::Result<()> {
    use glonax::transport::frame::FrameMessage;

    log::debug!("Starting host service");

    let mut client = glonax::transport::ConnectionOptions::new()
        .read(true)
        .connect("localhost:30051", "glonax-agent/0.1.0")
        .await
        .unwrap();

    log::info!("Connected to {}", "localhost:30051");

    client.send_request(FrameMessage::Instance).await.unwrap();

    let frame = client.read_frame().await.unwrap();
    match frame.message {
        FrameMessage::Null => {
            log::info!("Received null");
        }
        FrameMessage::Status => {
            let status = client
                .packet::<glonax::core::Status>(frame.payload_length)
                .await
                .unwrap();
            log::info!("Received status: {}", status);
        }
        FrameMessage::Instance => {
            let instance = client
                .packet::<glonax::core::Instance>(frame.payload_length)
                .await
                .unwrap();
            log::info!("Received instance: {}", instance);
        }
        _ => {}
    }

    client.send_request(FrameMessage::Status).await.unwrap();

    let frame = client.read_frame().await.unwrap();
    match frame.message {
        FrameMessage::Null => {
            log::info!("Received null");
        }
        FrameMessage::Status => {
            let status = client
                .packet::<glonax::core::Status>(frame.payload_length)
                .await
                .unwrap();
            log::info!("Received status: {}", status);
        }
        FrameMessage::Instance => {
            let instance = client
                .packet::<glonax::core::Instance>(frame.payload_length)
                .await
                .unwrap();
            log::info!("Received instance: {}", instance);
        }
        _ => {}
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    loop {
        client.send_request(FrameMessage::Pose).await.unwrap();
        // client.send_request(FrameMessage::Engine).await.unwrap();

        let frame = client.read_frame().await.unwrap();
        match frame.message {
            FrameMessage::Null => {
                log::info!("Received null");
            }
            FrameMessage::Status => {
                let status = client
                    .packet::<glonax::core::Status>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received status: {}", status);
            }
            FrameMessage::Instance => {
                let instance = client
                    .packet::<glonax::core::Instance>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received instance: {}", instance);
            }
            FrameMessage::Pose => {
                let pose = client
                    .packet::<glonax::core::Pose>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received pose: {}", pose);
            }
            FrameMessage::Engine => {
                let engine = client
                    .packet::<glonax::core::Engine>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received engine: {}", engine);
            }
            _ => {}
        }

        client.send_request(FrameMessage::Engine).await.unwrap();

        let frame = client.read_frame().await.unwrap();
        match frame.message {
            FrameMessage::Null => {
                log::info!("Received null");
            }
            FrameMessage::Status => {
                let status = client
                    .packet::<glonax::core::Status>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received status: {}", status);
            }
            FrameMessage::Instance => {
                let instance = client
                    .packet::<glonax::core::Instance>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received instance: {}", instance);
            }
            FrameMessage::Pose => {
                let pose = client
                    .packet::<glonax::core::Pose>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received pose: {}", pose);
            }
            FrameMessage::Engine => {
                let engine = client
                    .packet::<glonax::core::Engine>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received engine: {}", engine);
            }
            _ => {}
        }

        client.send_request(FrameMessage::Status).await.unwrap();

        let frame = client.read_frame().await.unwrap();
        match frame.message {
            FrameMessage::Null => {
                log::info!("Received null");
            }
            FrameMessage::Status => {
                let status = client
                    .packet::<glonax::core::Status>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received status: {}", status);
            }
            FrameMessage::Instance => {
                let instance = client
                    .packet::<glonax::core::Instance>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received instance: {}", instance);
            }
            FrameMessage::Pose => {
                let pose = client
                    .packet::<glonax::core::Pose>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received pose: {}", pose);
            }
            _ => {}
        }

        client.send_request(FrameMessage::VMS).await.unwrap();

        let frame = client.read_frame().await.unwrap();
        match frame.message {
            FrameMessage::Null => {
                log::info!("Received null");
            }
            FrameMessage::Status => {
                let status = client
                    .packet::<glonax::core::Status>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received status: {}", status);
            }
            FrameMessage::Instance => {
                let instance = client
                    .packet::<glonax::core::Instance>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received instance: {}", instance);
            }
            FrameMessage::Pose => {
                let pose = client
                    .packet::<glonax::core::Pose>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received pose: {}", pose);
            }
            FrameMessage::VMS => {
                let vms = client
                    .packet::<glonax::core::Host>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received vms: {}", vms);
            }
            _ => {}
        }

        client.send_request(FrameMessage::GNSS).await.unwrap();

        let frame = client.read_frame().await.unwrap();
        match frame.message {
            FrameMessage::Null => {
                log::info!("Received null");
            }
            FrameMessage::Status => {
                let status = client
                    .packet::<glonax::core::Status>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received status: {}", status);
            }
            FrameMessage::Instance => {
                let instance = client
                    .packet::<glonax::core::Instance>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received instance: {}", instance);
            }
            FrameMessage::Pose => {
                let pose = client
                    .packet::<glonax::core::Pose>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received pose: {}", pose);
            }
            FrameMessage::VMS => {
                let vms = client
                    .packet::<glonax::core::Host>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received vms: {}", vms);
            }
            FrameMessage::GNSS => {
                let gnss = client
                    .packet::<glonax::core::Gnss>(frame.payload_length)
                    .await
                    .unwrap();
                log::info!("Received gnss: {}", gnss);
            }
            _ => {}
        }

        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    // Ok(())
}
