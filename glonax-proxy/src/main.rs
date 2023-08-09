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
#[command(about = "Glonax proxy daemon", long_about = None)]
struct Args {
    /// Bind address.
    #[arg(short = 'b', long = "bind", default_value = "127.0.0.1:30051")]
    address: String,
    /// CAN network interface.
    interface: String,
    /// Refresh host service interval in milliseconds.
    #[arg(long, default_value_t = 500)]
    host_interval: u64,
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

    let mut config = config::ProxyConfig {
        address: args.address,
        interface: args.interface,
        host_interval: args.host_interval,
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

async fn daemonize(config: &config::ProxyConfig) -> anyhow::Result<()> {
    use tokio::net::TcpListener;
    use tokio::sync::broadcast::{self, Sender};

    log::info!("Starting proxy services");

    let (tx, _rx) = broadcast::channel(16);

    let (motion_tx, mut motion_rx) = tokio::sync::mpsc::channel(16);

    let host_sender: Sender<glonax::core::Signal> = tx.clone();
    let host_interval = config.host_interval;
    tokio::spawn(async move {
        use glonax::channel::SignalSource;

        log::debug!("Starting host service");

        let mut service = glonax::net::HostService::new();

        loop {
            service.refresh();

            let mut signals = vec![];
            service.collect_signals(&mut signals);

            for signal in signals {
                if let Err(e) = host_sender.send(signal) {
                    log::error!("Failed to send signal: {}", e);
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(host_interval)).await;
        }
    });

    let ecu_sender: Sender<glonax::core::Signal> = tx.clone();
    let ecu_interface = config.interface.clone();
    tokio::spawn(async move {
        use glonax::channel::SignalSource;
        use glonax::net::{EncoderService, J1939Network, Router};

        log::debug!("Starting ECU services");

        let network = J1939Network::new(&ecu_interface, 0x9E).unwrap();
        let mut router = Router::new(network);

        let mut encoder_list = vec![
            EncoderService::new(0x6A),
            EncoderService::new(0x6B),
            EncoderService::new(0x6C),
            EncoderService::new(0x6D),
        ];

        while router.listen().await.is_ok() {
            let mut signals = vec![];
            for encoder in &mut encoder_list {
                if let Some(message) = router.try_accept(encoder) {
                    message.collect_signals(&mut signals);
                }
            }

            for signal in signals {
                if let Err(e) = ecu_sender.send(signal) {
                    log::error!("Failed to send signal: {}", e);
                }
            }
        }

        log::debug!("ECU services shutdown");
    });

    let fifo_sender: Sender<glonax::core::Signal> = tx.clone();
    tokio::spawn(async move {
        log::debug!("Starting FIFO listener");

        let file = tokio::fs::OpenOptions::new()
            .read(true)
            .open("signal")
            .await
            .unwrap();

        let mut protocol = glonax::transport::Protocol::new(file);

        while let Ok(message) = protocol.read_frame().await {
            if let glonax::transport::Message::Signal(signal) = message {
                // log::debug!("Received signal: {}", signal);

                if let Err(e) = fifo_sender.send(signal) {
                    log::error!("Failed to send signal: {}", e);
                }
            }
        }
    });

    let ecu_interface = config.interface.clone();
    tokio::spawn(async move {
        use glonax::net::{ActuatorService, J1939Network};

        log::debug!("Starting motion listener");

        let network = J1939Network::new(&ecu_interface, 0x9E).unwrap();

        let service = ActuatorService::new(0x4A);

        while let Some(motion) = motion_rx.recv().await {
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
                                    .map(|changeset| (changeset.actuator as u8, changeset.value))
                                    .collect(),
                            ),
                        )
                        .await
                        .unwrap();
                }
            }
        }
    });

    let listener = TcpListener::bind(config.address.clone()).await?;

    loop {
        let (stream, addr) = listener.accept().await?;

        log::info!("Accepted connection from: {}", addr);

        let (stream_reader, stream_writer) = stream.into_split();

        let mut rx = tx.subscribe();

        let session_motion_tx = motion_tx.clone();

        tokio::spawn(async move {
            let mut protocol_out = glonax::transport::Protocol::new(stream_writer);

            while let Ok(signal) = rx.recv().await {
                if let Err(_e) = protocol_out.write_frame6(signal).await {
                    // log::error!("Failed to write to socket: {}", e);
                    break;
                }
            }

            log::info!("Signal listener shutdown");
        });

        tokio::spawn(async move {
            let mut session_name = String::new();

            let mut protocol_in = glonax::transport::Protocol::new(stream_reader);

            while let Ok(message) = protocol_in.read_frame().await {
                match message {
                    glonax::transport::Message::Start(session) => {
                        log::info!("Session started for: {}", session.name());
                        session_name = session.name().to_string();
                    }
                    glonax::transport::Message::Shutdown => {
                        log::info!("Session shutdown for: {}", session_name);
                        break;
                    }
                    glonax::transport::Message::Motion(motion) => {
                        log::debug!("Received motion: {}", motion);

                        if let Err(e) = session_motion_tx.send(motion).await {
                            log::error!("Failed to send motion: {}", e);
                        }
                    }
                    _ => {}
                }
            }

            log::info!("Connection closed for: {}", addr);
        });
    }

    // log::debug!("{} was shutdown gracefully", config.global.bin_name);

    // Ok(())
}
