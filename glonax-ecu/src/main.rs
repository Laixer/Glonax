// Copyright (C) 2023 Laixer Equipment B.V.
// All rights reserved.
//
// This software may be modified and distributed under the terms
// of the included license.  See the LICENSE file for details.

use clap::Parser;

mod config;

const DEVICE_NET_LOCAL_ADDR: u8 = 0x9e;

#[derive(Parser)]
#[command(author = "Copyright (C) 2023 Laixer Equipment B.V.")]
#[command(version, propagate_version = true)]
#[command(about = "Glonax ECU daemon", long_about = None)]
struct Args {
    /// Bind address.
    #[arg(short = 'b', long = "bind", default_value = "[::1]:50051")]
    address: String,
    /// CAN network interface.
    interface: String,
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

    let bin_name = env!("CARGO_BIN_NAME");

    let mut config = config::EcuConfig {
        address: args.address,
        interface: args.interface,
        global: glonax::GlobalConfig::default(),
    };

    config.global.bin_name = bin_name.to_string();
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

async fn daemonize(config: &config::EcuConfig) -> anyhow::Result<()> {
    let runtime = glonax::RuntimeBuilder::from_config(config)?
        .with_shutdown()
        .build();

    let interface = config.interface.clone();
    runtime.spawn_background_task(async move {
        use glonax::channel::SignalSource;
        use glonax::net::{EncoderService, EngineManagementSystem, J1939Network, Router};

        log::info!("Starting controller units services");

        let network = J1939Network::new(&interface, DEVICE_NET_LOCAL_ADDR).unwrap();

        let mut router = Router::new(network);

        let mut engine_management_service = EngineManagementSystem::new(0x0);
        let mut encoder_list = vec![
            EncoderService::new(0x6A),
            EncoderService::new(0x6B),
            EncoderService::new(0x6C),
            EncoderService::new(0x6D),
        ];

        loop {
            log::debug!("Waiting for FIFO connection: {}", "signal");

            let file = tokio::fs::OpenOptions::new()
                .write(true)
                .open("signal")
                .await
                .unwrap();

            log::debug!("Connected to FIFO: {}", "signal");

            let mut protocol = glonax::transport::Protocol::new(file);

            while router.listen().await.is_ok() {
                let mut signals = vec![];
                if let Some(message) = router.try_accept(&mut engine_management_service) {
                    message.collect_signals(&mut signals);
                }

                for encoder in &mut encoder_list {
                    if let Some(message) = router.try_accept(encoder) {
                        message.collect_signals(&mut signals);
                    }
                }

                if let Err(e) = protocol.write_all6(signals).await {
                    log::error!("Failed to write to socket: {}", e);
                    break;
                }
            }
        }
    });

    let interface = config.interface.clone();
    runtime.spawn_background_task(async move {
        use glonax::net::{ActuatorService, J1939Network};

        log::info!("Starting motion listener");

        let network = J1939Network::new(&interface, DEVICE_NET_LOCAL_ADDR).unwrap();

        let service = ActuatorService::new(0x4A);

        loop {
            log::debug!("Waiting for FIFO connection: {}", "motion");

            let file = tokio::fs::OpenOptions::new()
                .read(true)
                .open("motion")
                .await
                .unwrap();

            log::debug!("Connected to FIFO: {}", "motion");

            let mut protocol = glonax::transport::Protocol::new(file);

            while let Ok(message) = protocol.read_frame().await {
                if let glonax::transport::Message::Motion(motion) = message {
                    log::debug!("Received motion: {}", motion);

                    match motion {
                        glonax::core::Motion::StopAll => {
                            network.send_vectored(&service.lock()).await.unwrap();
                        }
                        glonax::core::Motion::ResumeAll => {
                            network.send_vectored(&service.unlock()).await.unwrap();
                        }
                        glonax::core::Motion::StraightDrive(_value) => {
                            // network
                            //     .send_vectored(&service.drive_command(0, value))
                            //     .await
                            //     .unwrap();
                        }
                        glonax::core::Motion::Change(changes) => {
                            network
                                .send_vectored(
                                    &service.actuator_command(
                                        changes
                                            .iter()
                                            .map(|changeset| {
                                                (changeset.actuator as u8, changeset.value)
                                            })
                                            .collect(),
                                    ),
                                )
                                .await
                                .unwrap();
                        }
                    }
                } else {
                    // TODO: Which message was received?
                    log::warn!("Received non-motion message");
                }
            }
        }
    });

    runtime.wait_for_shutdown().await;

    log::debug!("{} was shutdown gracefully", config.global.bin_name);

    Ok(())
}
