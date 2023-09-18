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
    #[arg(short = 'b', long = "bind", default_value = "0.0.0.0:30051")]
    address: String,
    /// CAN network interface.
    interface: String,
    /// CAN network interface.
    interface2: Option<String>,
    /// Refresh host service interval in milliseconds.
    #[arg(long, default_value_t = 500)]
    host_interval: u64,
    /// Configuration file.
    #[arg(long = "config", default_value = "/etc/glonax.conf")]
    config: std::path::PathBuf,
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
        interface2: args.interface2,
        host_interval: args.host_interval,
        instance: glonax::from_toml(args.config)?,
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

    log::info!("Starting proxy services");

    log::info!("Instance ID: {}", config.instance.id);
    log::info!("Instance Model: {}", config.instance.model);
    log::info!("Instance Name: {}", config.instance.name);

    if config.instance.id.starts_with("00000000") {
        log::warn!("Instance ID is not set or invalid");
    }

    let (signal_tx, signal_rx) = tokio::sync::mpsc::channel(16);
    let (motion_tx, mut motion_rx) = tokio::sync::mpsc::channel(16);

    let host_sender = signal_tx.clone();
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
                if let Err(e) = host_sender.send(signal).await {
                    log::error!("Failed to send signal: {}", e);
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(host_interval)).await;
        }
    });

    let ecu_sender = signal_tx.clone();
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

        loop {
            if let Err(e) = router.listen().await {
                log::error!("Failed to receive from router: {}", e);
            }

            let mut signals = vec![];
            for encoder in &mut encoder_list {
                if let Some(message) = router.try_accept(encoder) {
                    message.collect_signals(&mut signals);
                }
            }

            for signal in signals {
                if let Err(e) = ecu_sender.send(signal).await {
                    log::error!("Failed to send signal: {}", e);
                }
            }
        }
    });

    let ecu_sender = signal_tx.clone();
    if let Some(ecu_interface) = config.interface2.clone() {
        tokio::spawn(async move {
            use glonax::channel::SignalSource;
            use glonax::net::{EngineManagementSystem, J1939Network, Router};

            log::debug!("Starting EMS service");

            let network = J1939Network::new(&ecu_interface, 0x9E).unwrap();
            let mut router = Router::new(network);

            let mut engine_management_service = EngineManagementSystem::new(0x0);

            loop {
                if let Err(e) = router.listen().await {
                    log::error!("Failed to receive from router: {}", e);
                }

                let mut signals = vec![];
                if let Some(message) = router.try_accept(&mut engine_management_service) {
                    message.collect_signals(&mut signals);
                }

                for signal in signals {
                    if let Err(e) = ecu_sender.send(signal).await {
                        log::error!("Failed to send signal: {}", e);
                    }
                }
            }
        });
    }

    let fifo_sender = signal_tx.clone();
    tokio::spawn(async move {
        loop {
            log::debug!("Waiting for FIFO connection: signal");

            let mut client = glonax::transport::Client::open_read("signal")
                .await
                .unwrap();

            log::debug!("Connected to FIFO: signal");

            while let Ok(signal) = client.recv_signal().await {
                if let Err(e) = fifo_sender.send(signal).await {
                    log::error!("Failed to send signal: {}", e);
                }
            }

            log::debug!("FIFO listener shutdown: signal");
        }
    });

    let ecu_interface = config.interface.clone();
    tokio::spawn(async move {
        use glonax::core::Motion;
        use glonax::net::{ActuatorService, J1939Network};

        log::debug!("Starting motion listener");

        let network = J1939Network::new(&ecu_interface, 0x9E).unwrap();

        let service = ActuatorService::new(0x4A);

        while let Some(motion) = motion_rx.recv().await {
            match motion {
                Motion::StopAll => {
                    if let Err(e) = network.send_vectored(&service.lock()).await {
                        log::error!("Failed to send motion: {}", e);
                    }
                }
                Motion::ResumeAll => {
                    if let Err(e) = network.send_vectored(&service.unlock()).await {
                        log::error!("Failed to send motion: {}", e);
                    }
                }
                Motion::ResetAll => {
                    if let Err(e) = network.send_vectored(&service.lock()).await {
                        log::error!("Failed to send motion: {}", e);
                    }
                    if let Err(e) = network.send_vectored(&service.unlock()).await {
                        log::error!("Failed to send motion: {}", e);
                    }
                }
                Motion::StraightDrive(value) => {
                    let frames = &service.drive_straight(value);

                    if let Err(e) = network.send_vectored(frames).await {
                        log::error!("Failed to send motion: {}", e);
                    }
                }
                Motion::Change(changes) => {
                    let frames = &service.actuator_command(
                        changes
                            .iter()
                            .map(|changeset| (changeset.actuator as u8, changeset.value))
                            .collect(),
                    );

                    if let Err(e) = network.send_vectored(frames).await {
                        log::error!("Failed to send motion: {}", e);
                    }
                }
            }
        }

        log::debug!("Motion listener shutdown");
    });

    let instance_id = config.instance.id.clone();
    let instance_model = config.instance.model.clone();
    let instance_name = config.instance.name.clone();

    let mut session_signal_rx = signal_rx;
    tokio::spawn(async move {
        use glonax::core::Metric;
        use glonax::transport::frame::{Frame, FrameMessage};
        use std::time::Instant;

        log::debug!("Starting signal broadcast");

        let socket = glonax::channel::any_bind().await.unwrap();

        let mut now = Instant::now();

        let broadcast_addr = std::net::SocketAddrV4::new(
            std::net::Ipv4Addr::BROADCAST,
            glonax::constants::DEFAULT_NETWORK_PORT,
        );

        let mut signal_gnss_timeout = Instant::now();
        let mut signal_encoder_0x6a_timeout = Instant::now();
        let mut signal_encoder_0x6b_timeout = Instant::now();
        let mut signal_encoder_0x6c_timeout = Instant::now();
        let mut signal_encoder_0x6d_timeout = Instant::now();
        let mut signal_engine_timeout = Instant::now();

        let mut status = glonax::core::Status::Healthy;

        while let Some(signal) = session_signal_rx.recv().await {
            match signal.metric {
                Metric::VmsMemoryUsage((memory_used, memory_total)) => {
                    let memory_usage = (memory_used as f64 / memory_total as f64) * 100.0;

                    if memory_usage > 90.0 {
                        log::warn!("Memory usage is above 90%: {:.2}%", memory_usage);
                        status = glonax::core::Status::DegradedHighUsageMemory;
                    }
                }
                Metric::GnssSatellites(_) => {
                    signal_gnss_timeout = Instant::now();
                }
                Metric::EncoderAbsAngle((node, _)) => match node {
                    0x6A => {
                        signal_encoder_0x6a_timeout = Instant::now();
                    }
                    0x6B => {
                        signal_encoder_0x6b_timeout = Instant::now();
                    }
                    0x6C => {
                        signal_encoder_0x6c_timeout = Instant::now();
                    }
                    0x6D => {
                        signal_encoder_0x6d_timeout = Instant::now();
                    }
                    _ => {}
                },
                Metric::EngineRpm(_) => {
                    signal_engine_timeout = Instant::now();
                }
                _ => {}
            }

            if signal_gnss_timeout.elapsed().as_secs() > 5 {
                log::warn!("GNSS signal timeout: no update in last 5 seconds");
                status = glonax::core::Status::DegradedTimeoutGNSS;
                signal_gnss_timeout = Instant::now();
            }
            if signal_encoder_0x6a_timeout.elapsed().as_secs() > 1 {
                log::warn!("Encoder 0x6A signal timeout: no update in last 1 second");
                status = glonax::core::Status::DegradedTimeoutEncoder;
                signal_encoder_0x6a_timeout = Instant::now();
            }
            if signal_encoder_0x6b_timeout.elapsed().as_secs() > 1 {
                log::warn!("Encoder 0x6B signal timeout: no update in last 1 second");
                status = glonax::core::Status::DegradedTimeoutEncoder;
                signal_encoder_0x6b_timeout = Instant::now();
            }
            if signal_encoder_0x6c_timeout.elapsed().as_secs() > 1 {
                log::warn!("Encoder 0x6C signal timeout: no update in last 1 second");
                status = glonax::core::Status::DegradedTimeoutEncoder;
                signal_encoder_0x6c_timeout = Instant::now();
            }
            if signal_encoder_0x6d_timeout.elapsed().as_secs() > 1 {
                log::warn!("Encoder 0x6D signal timeout: no update in last 1 second");
                status = glonax::core::Status::DegradedTimeoutEncoder;
                signal_encoder_0x6d_timeout = Instant::now();
            }
            if signal_engine_timeout.elapsed().as_secs() > 5 {
                log::warn!("Engine signal timeout: no update in last 5 seconds");
                status = glonax::core::Status::DegradedTimeoutEngine;
                signal_engine_timeout = Instant::now();
            }

            let payload = signal.to_bytes();

            let mut frame = Frame::new(FrameMessage::Signal, payload.len());
            frame.put(&payload[..]);

            if let Err(e) = socket.send_to(frame.as_ref(), broadcast_addr).await {
                log::error!("Failed to send signal: {}", e);
                break;
            }

            if now.elapsed().as_millis() > 1_000 {
                {
                    let instance = glonax::core::Instance::new(
                        instance_id.clone(),
                        instance_model.clone(),
                        instance_name.clone(),
                    );
                    let payload = instance.to_bytes();

                    let mut frame = Frame::new(FrameMessage::Instance, payload.len());
                    frame.put(&payload[..]);

                    if let Err(e) = socket.send_to(frame.as_ref(), broadcast_addr).await {
                        log::error!("Failed to send signal: {}", e);
                    }
                }

                {
                    let payload = status.to_bytes();

                    let mut frame = Frame::new(FrameMessage::Status, payload.len());
                    frame.put(&payload[..]);

                    if let Err(e) = socket.send_to(frame.as_ref(), broadcast_addr).await {
                        log::error!("Failed to send signal: {}", e);
                    }
                }

                {
                    status = glonax::core::Status::Healthy;
                    now = Instant::now();
                }
            }
        }

        log::debug!("Signal broadcast shutdown");
    });

    motion_tx.send(glonax::core::Motion::ResetAll).await?;

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(10));

    log::debug!("Waiting for connection to {}", config.address);

    let listener = TcpListener::bind(config.address.clone()).await?;

    loop {
        let (stream, addr) = listener.accept().await?;

        let permit = match semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                log::warn!("Too many connections");
                continue;
            }
        };

        let session_motion_tx = motion_tx.clone();
        tokio::spawn(async move {
            log::debug!("Accepted connection from: {}", addr);

            let mut client = glonax::transport::Client::new(stream);

            // TODO: Handle errors
            // TODO: Set timeout
            let start = client
                .recv_start()
                .await
                .expect("Failed to receive start message");

            let session_name = start.name().to_string();
            let session_write = start.is_write();
            let session_failsafe = start.is_failsafe();
            let mut session_shutdown = false;

            log::info!("Session started for: {}", session_name);

            while let Ok(frame) = client.read_frame().await {
                match frame.message {
                    glonax::transport::frame::FrameMessage::Shutdown => {
                        log::debug!("Client requested shutdown");
                        session_shutdown = true;
                        break;
                    }
                    glonax::transport::frame::FrameMessage::Motion => {
                        if session_write {
                            let motion = client.motion(frame.payload_length).await.unwrap();

                            if let Err(e) = session_motion_tx.send(motion).await {
                                log::error!("Failed to send motion: {}", e);
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }

            if !session_shutdown && session_write && session_failsafe {
                log::warn!("Enacting failsafe for: {}", session_name);

                if let Err(e) = session_motion_tx.send(glonax::core::Motion::StopAll).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }

            log::info!("Session shutdown for: {}", session_name);

            drop(permit);
        });
    }

    // log::debug!("{} was shutdown gracefully", config.global.bin_name);

    // Ok(())
}
