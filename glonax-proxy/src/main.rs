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
    /// Configuration file.
    #[arg(long = "config", default_value = "/etc/glonax.conf")]
    config: String,
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
        instance: glonax::instance_config(args.config)?,
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
    use tokio::net::{TcpListener, UdpSocket};
    use tokio::sync::broadcast::{self, Sender};

    log::info!("Starting proxy services");
    log::debug!("Instance ID: {}", config.instance.instance);
    log::debug!("Instance model: {}", config.instance.model);
    log::debug!("Instance name: {}", config.instance.name);

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

        // log::debug!("Host service shutdown");
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
        loop {
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

            log::debug!("FIFO listener shutdown");
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
                glonax::core::Motion::StraightDrive(value) => {
                    network
                        .send_vectored(&service.drive_straight(value))
                        .await
                        .unwrap();
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

        log::debug!("Motion listener shutdown");
    });

    let instance = config.instance.instance.clone();
    let model = config.instance.model.clone();
    let name = config.instance.name.clone();

    let mut session_signal_rx = tx.subscribe();
    tokio::spawn(async move {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        socket.set_broadcast(true).unwrap();

        let broadcast_addr = std::net::SocketAddrV4::new(
            std::net::Ipv4Addr::BROADCAST,
            glonax::constants::DEFAULT_NETWORK_PORT,
        );

        let ps = glonax::transport::frame::ProxyService::new(
            instance.clone(),
            model.clone(),
            name.clone(),
        );
        let payload = ps.to_bytes();

        let mut frame = glonax::transport::frame::Frame::new(
            glonax::transport::frame::FrameMessage::Instance,
            payload.len(),
        );
        frame.put(&payload[..]);

        if let Err(e) = socket.send_to(frame.as_ref(), broadcast_addr).await {
            log::error!("Failed to send signal: {}", e);
        }

        let mut now = std::time::Instant::now();
        while let Ok(signal) = session_signal_rx.recv().await {
            let payload = signal.to_bytes();

            let mut frame = glonax::transport::frame::Frame::new(
                glonax::transport::frame::FrameMessage::Signal,
                payload.len(),
            );
            frame.put(&payload[..]);

            if let Err(e) = socket.send_to(frame.as_ref(), broadcast_addr).await {
                log::error!("Failed to send signal: {}", e);
                break;
            }

            if now.elapsed().as_millis() > 1_000 {
                let ps = glonax::transport::frame::ProxyService::new(
                    instance.clone(),
                    model.clone(),
                    name.clone(),
                );
                let payload = ps.to_bytes();

                let mut frame = glonax::transport::frame::Frame::new(
                    glonax::transport::frame::FrameMessage::Instance,
                    payload.len(),
                );
                frame.put(&payload[..]);

                if let Err(e) = socket.send_to(frame.as_ref(), broadcast_addr).await {
                    log::error!("Failed to send signal: {}", e);
                } else {
                    now = std::time::Instant::now();
                }
            }
        }
    });

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(10));

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
        // let mut session_signal_rx = tx.subscribe();

        let instance = config.instance.instance.clone();
        let model = config.instance.model.clone();
        let name = config.instance.name.clone();

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
            let session_name2 = start.name().to_string();

            // let session_read = start.is_read();
            let session_write = start.is_write();
            let session_failsafe = start.is_failsafe();

            log::info!("Session started for: {}", session_name);

            client.send_instance(instance, model, name).await.unwrap();

            let (mut client_reader, mut _client_writer) = client.into_split();

            let session_teardown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

            // let session_teardown_r = session_teardown.clone();

            // let reader_handle = tokio::spawn(async move {
            //     if session_read {
            //         log::debug!("Session reader started for: {}", session_name);

            //         while let Ok(signal) = session_signal_rx.recv().await {
            //             if session_teardown_r.load(std::sync::atomic::Ordering::Relaxed) {
            //                 break;
            //             }

            //             if let Err(e) = client_writer.send_signal(signal).await {
            //                 log::error!("Failed to send signal: {}", e);
            //                 break;
            //             }
            //         }

            //         log::debug!("Session reader shutdown for: {}", session_name);
            //     }
            // });

            tokio::spawn(async move {
                log::debug!("Session writer started for: {}", session_name2);

                while let Ok(frame) = client_reader.read_frame().await {
                    match frame.message {
                        glonax::transport::frame::FrameMessage::Null => {}
                        glonax::transport::frame::FrameMessage::Start => {}
                        glonax::transport::frame::FrameMessage::Shutdown => {
                            log::debug!("Client requested shutdown");
                            session_teardown.store(true, std::sync::atomic::Ordering::Relaxed);
                            break;
                        }
                        glonax::transport::frame::FrameMessage::Motion => {
                            if session_write {
                                let payload_buffer = &mut vec![0u8; frame.payload_length];

                                use tokio::io::AsyncReadExt;

                                let k = client_reader.inner_mut();

                                k.read_exact(payload_buffer).await.unwrap();

                                // match glonax::core::Motion::try_from(&payload_buffer[..]) {
                                //     Ok(motion) => {
                                //         return Ok(motion);
                                //     }
                                //     Err(_) => {
                                //         log::warn!("Invalid motion payload");
                                //         return Err(std::io::Error::new(
                                //             std::io::ErrorKind::InvalidData,
                                //             "Invalid motion payload",
                                //         ));
                                //     }
                                // }

                                // log::debug!("Received motion: {}", frame.payload_length);
                                if let Err(e) = session_motion_tx
                                    .send(
                                        glonax::core::Motion::try_from(&payload_buffer[..])
                                            .expect("Failed to parse motion"),
                                    )
                                    .await
                                {
                                    log::error!("Failed to send motion: {}", e);
                                    break;
                                }
                            }
                        }
                        _ => {} // glonax::transport::frame::FrameMessage::Signal => {}

                                // glonax::transport::Message::Null => {}
                                // glonax::transport::Message::Start(_) => {}
                                // glonax::transport::Message::Shutdown => {
                                //     log::debug!("Client requested shutdown");
                                //     session_teardown.store(true, std::sync::atomic::Ordering::Relaxed);
                                //     break;
                                // }
                                // glonax::transport::Message::Motion(motion) => {
                                //     if session_write {
                                //         log::debug!("Received motion: {}", motion);
                                //         if let Err(e) = session_motion_tx.send(motion).await {
                                //             log::error!("Failed to send motion: {}", e);
                                //             break;
                                //         }
                                //     }
                                // }
                                // glonax::transport::Message::Signal(_) => {}
                    }
                }

                log::debug!("Session writer shutdown for: {}", session_name2);

                let q = session_teardown.compare_exchange(
                    false,
                    true,
                    std::sync::atomic::Ordering::Acquire,
                    std::sync::atomic::Ordering::Relaxed,
                );

                if q.is_ok() {
                    log::warn!("Session was shutdown by other circumstances");

                    if session_write && session_failsafe {
                        log::warn!("Enacting failsafe for: {}", session_name2);
                        session_motion_tx
                            .send(glonax::core::Motion::StopAll)
                            .await
                            .unwrap();
                    }
                }

                // reader_handle.await.unwrap();

                log::info!("Session shutdown for: {}", session_name2);

                drop(permit);
            });
        });
    }

    // log::debug!("{} was shutdown gracefully", config.global.bin_name);

    // Ok(())
}
