use glonax::core::{Motion, Signal};
use tokio::sync::mpsc;

use crate::config::ProxyConfig;

pub type SignalSender = tokio::sync::mpsc::Sender<glonax::core::Signal>;
pub type MotionSender = tokio::sync::mpsc::Sender<glonax::core::Motion>;
pub type SharedMachineState = std::sync::Arc<tokio::sync::RwLock<glonax::MachineState>>;

pub(super) async fn service_host(local_config: ProxyConfig, local_sender: SignalSender) {
    use glonax::channel::SignalSource;

    log::debug!("Starting host service");

    let mut service = glonax::net::HostService::new();

    loop {
        service.refresh();

        let mut signals = vec![];
        service.collect_signals(&mut signals);

        for signal in signals {
            if let Err(e) = local_sender.send(signal).await {
                log::error!("Failed to send signal: {}", e);
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(local_config.host_interval)).await;
    }
}

pub(super) async fn service_fifo(_local_config: ProxyConfig, local_sender: SignalSender) {
    log::debug!("Starting FIFO service");

    loop {
        log::debug!("Waiting for FIFO connection: signal");

        match glonax::transport::Client::open_read("signal").await {
            Ok(mut client) => {
                log::debug!("Connected to FIFO: signal");

                while let Ok(signal) = client.recv_signal().await {
                    if let Err(e) = local_sender.send(signal).await {
                        log::error!("Failed to send signal: {}", e);
                    }
                }

                log::debug!("FIFO listener shutdown: signal");
            }
            Err(e) => {
                log::error!("Failed to connect to FIFO: signal: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

pub(super) async fn service_net_encoder(local_config: ProxyConfig, local_sender: SignalSender) {
    use glonax::channel::SignalSource;
    use glonax::net::{EncoderService, J1939Network, Router};

    log::debug!("Starting encoder services");

    match J1939Network::new(
        &local_config.interface,
        glonax::consts::DEFAULT_J1939_ADDRESS,
    ) {
        Ok(network) => {
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
                    if let Err(e) = local_sender.send(signal).await {
                        log::error!("Failed to send signal: {}", e);
                    }
                }
            }
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}

pub(super) async fn service_net_ems(local_config: ProxyConfig, local_sender: SignalSender) {
    if local_config.interface2.is_none() {
        return;
    }

    use glonax::channel::SignalSource;
    use glonax::net::{EngineManagementSystem, J1939Network, Router};

    log::debug!("Starting EMS service");

    match J1939Network::new(
        &local_config.interface2.unwrap(),
        glonax::consts::DEFAULT_J1939_ADDRESS,
    ) {
        Ok(network) => {
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
                    if let Err(e) = local_sender.send(signal).await {
                        log::error!("Failed to send signal: {}", e);
                    }
                }
            }
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}

pub(super) async fn service_gnss(_local_config: ProxyConfig, local_sender: SignalSender) {
    use glonax::channel::SignalSource;
    use tokio::io::{AsyncBufReadExt, BufReader};

    log::debug!("Starting GNSS service");

    loop {
        match glonax_serial::Uart::open(
            &std::path::Path::new("/dev/ttyUSB0"),
            glonax_serial::BaudRate::from_speed(9600),
        ) {
            Ok(serial) => {
                let reader = BufReader::new(serial);
                let mut lines = reader.lines();

                let service = glonax::net::NMEAService::new();

                while let Ok(Some(line)) = lines.next_line().await {
                    if let Some(message) = service.decode(line) {
                        log::trace!("Received message: {}", message);

                        let mut signals = vec![];
                        message.collect_signals(&mut signals);

                        for signal in signals {
                            if let Err(e) = local_sender.send(signal).await {
                                log::error!("Failed to send signal: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to open serial: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

pub(super) async fn sink_proxy(
    local_config: ProxyConfig,
    local_machine_state: SharedMachineState,
    mut signal_rx: mpsc::Receiver<Signal>,
) {
    use glonax::core::Metric;
    use glonax::transport::frame::{Frame, FrameMessage};
    use std::time::Instant;

    log::debug!("Starting signal broadcast");

    let socket = glonax::channel::any_bind().await.unwrap();

    let mut now = Instant::now();

    let broadcast_addr = std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::BROADCAST,
        glonax::consts::DEFAULT_NETWORK_PORT,
    );

    let mut signal_gnss_timeout = Instant::now();
    let mut signal_encoder_0x6a_timeout = Instant::now();
    let mut signal_encoder_0x6b_timeout = Instant::now();
    let mut signal_encoder_0x6c_timeout = Instant::now();
    let mut signal_encoder_0x6d_timeout = Instant::now();
    let mut signal_engine_timeout = Instant::now();

    while let Some(signal) = signal_rx.recv().await {
        match signal.metric {
            Metric::VmsUptime(uptime) => {
                local_machine_state.write().await.data.uptime = Some(uptime);
            }
            Metric::VmsMemoryUsage((memory_used, memory_total)) => {
                let memory_usage = (memory_used as f64 / memory_total as f64) * 100.0;

                local_machine_state.write().await.data.memory = Some(memory_usage as u64);

                if memory_usage > 90.0 {
                    log::warn!("Memory usage is above 90%: {:.2}%", memory_usage);
                    local_machine_state.write().await.status =
                        glonax::core::Status::DegradedHighUsageMemory;
                }
            }
            Metric::VmsSwapUsage((swap_used, swap_total)) => {
                let swap_usage = (swap_used as f64 / swap_total as f64) * 100.0;

                local_machine_state.write().await.data.swap = Some(swap_usage as u64);
            }
            Metric::VmsCpuLoad((cpu_load_1, cpu_load_5, cpu_load_15)) => {
                local_machine_state.write().await.data.cpu_load =
                    Some((cpu_load_1, cpu_load_5, cpu_load_15));
            }
            Metric::GnssLatLong(lat_long) => {
                local_machine_state.write().await.data.location = Some(lat_long);
            }
            Metric::GnssAltitude(altitude) => {
                local_machine_state.write().await.data.altitude = Some(altitude);
            }
            Metric::GnssSpeed(speed) => {
                local_machine_state.write().await.data.speed = Some(speed);
            }
            Metric::GnssHeading(heading) => {
                local_machine_state.write().await.data.heading = Some(heading);
            }
            Metric::GnssSatellites(satellites) => {
                local_machine_state.write().await.data.satellites = Some(satellites);

                signal_gnss_timeout = Instant::now();
            }
            Metric::EngineRpm(rpm) => {
                local_machine_state.write().await.data.rpm = Some(rpm);

                signal_engine_timeout = Instant::now();
            }
            Metric::EncoderAbsAngle((node, value)) => match node {
                0x6A => {
                    local_machine_state
                        .write()
                        .await
                        .data
                        .encoders
                        .insert(0x6A, value as i16);

                    signal_encoder_0x6a_timeout = Instant::now();
                }
                0x6B => {
                    local_machine_state
                        .write()
                        .await
                        .data
                        .encoders
                        .insert(0x6B, value as i16);

                    signal_encoder_0x6b_timeout = Instant::now();
                }
                0x6C => {
                    local_machine_state
                        .write()
                        .await
                        .data
                        .encoders
                        .insert(0x6C, value as i16);

                    signal_encoder_0x6c_timeout = Instant::now();
                }
                0x6D => {
                    local_machine_state
                        .write()
                        .await
                        .data
                        .encoders
                        .insert(0x6D, value as i16);

                    signal_encoder_0x6d_timeout = Instant::now();
                }
                _ => {}
            },
            _ => {}
        }

        if signal_gnss_timeout.elapsed().as_secs() > 5 {
            log::warn!("GNSS signal timeout: no update in last 5 seconds");
            local_machine_state.write().await.status = glonax::core::Status::DegradedTimeoutGNSS;
            signal_gnss_timeout = Instant::now();
        }
        if signal_encoder_0x6a_timeout.elapsed().as_secs() > 1 {
            log::warn!("Encoder 0x6A signal timeout: no update in last 1 second");
            local_machine_state.write().await.status = glonax::core::Status::DegradedTimeoutEncoder;
            signal_encoder_0x6a_timeout = Instant::now();
        }
        if signal_encoder_0x6b_timeout.elapsed().as_secs() > 1 {
            log::warn!("Encoder 0x6B signal timeout: no update in last 1 second");
            local_machine_state.write().await.status = glonax::core::Status::DegradedTimeoutEncoder;
            signal_encoder_0x6b_timeout = Instant::now();
        }
        if signal_encoder_0x6c_timeout.elapsed().as_secs() > 1 {
            log::warn!("Encoder 0x6C signal timeout: no update in last 1 second");
            local_machine_state.write().await.status = glonax::core::Status::DegradedTimeoutEncoder;
            signal_encoder_0x6c_timeout = Instant::now();
        }
        if signal_encoder_0x6d_timeout.elapsed().as_secs() > 1 {
            log::warn!("Encoder 0x6D signal timeout: no update in last 1 second");
            local_machine_state.write().await.status = glonax::core::Status::DegradedTimeoutEncoder;
            signal_encoder_0x6d_timeout = Instant::now();
        }
        if signal_engine_timeout.elapsed().as_secs() > 5 {
            log::warn!("Engine signal timeout: no update in last 5 seconds");
            local_machine_state.write().await.status = glonax::core::Status::DegradedTimeoutEngine;
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
            // TODO: Remove
            {
                let instance = glonax::core::Instance::new(
                    local_config.instance.id.clone(),
                    local_config.instance.model.clone(),
                    local_config.instance.name.clone(),
                );
                let payload = instance.to_bytes();

                let mut frame = Frame::new(FrameMessage::Instance, payload.len());
                frame.put(&payload[..]);

                if let Err(e) = socket.send_to(frame.as_ref(), broadcast_addr).await {
                    log::error!("Failed to send signal: {}", e);
                }
            }

            {
                local_machine_state.write().await.status = glonax::core::Status::Healthy;
                now = Instant::now();
            }
        }
    }

    log::debug!("Signal broadcast shutdown");
}

pub(super) async fn service_remote_probe(
    _local_config: ProxyConfig,
    local_machine_state: SharedMachineState,
) {
    log::debug!("Starting host service");

    // let url = reqwest::Url::parse(HOST).unwrap();

    // let client = reqwest::Client::builder()
    //     .user_agent("glonax-agent/0.1.0")
    //     .timeout(std::time::Duration::from_secs(5))
    //     .https_only(true)
    //     .build()
    //     .unwrap();

    // let request_url = url
    //     .join(&format!("api/v1/{}/probe", config.instance.id))
    //     .unwrap();

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(14)).await;

        // if config.probe {
        //     let data = telemetrics.read().await;

        //     if data.status.is_none() {
        //         continue;
        //     }

        //     let response = client
        //         .post(request_url.clone())
        //         .json(&*data)
        //         .send()
        //         .await
        //         .unwrap();

        //     if response.status() == 200 {
        //         log::info!("Probe sent successfully");
        //     } else {
        //         log::error!("Probe failed, status: {}", response.status());
        //     }
        // };

        log::trace!("{}", local_machine_state.read().await.data);
    }
}

pub(super) async fn sink_net_actuator(
    local_config: ProxyConfig,
    mut motion_rx: mpsc::Receiver<Motion>,
) {
    use glonax::net::{ActuatorService, J1939Network};

    log::debug!("Starting motion listener");

    match J1939Network::new(
        &local_config.interface,
        glonax::consts::DEFAULT_J1939_ADDRESS,
    ) {
        Ok(network) => {
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
        }
        Err(e) => log::error!("Failed to create network: {}", e),
    }
}

pub(super) async fn service_remote_server(
    local_config: ProxyConfig,
    local_machine_state: SharedMachineState,
    local_sender: MotionSender,
    _shutdown: tokio::sync::broadcast::Receiver<()>,
) {
    use glonax::transport::frame::FrameMessage;
    use tokio::net::TcpListener;

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
        glonax::consts::NETWORK_MAX_CLIENTS,
    ));

    log::debug!("Waiting for connection to {}", local_config.address);
    let listener = TcpListener::bind(local_config.address.clone())
        .await
        .unwrap();

    loop {
        let (stream, addr) = listener.accept().await.unwrap();

        let permit = match semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                log::warn!("Too many connections");
                continue;
            }
        };

        let local_config = local_config.clone();
        let local_machine_state = local_machine_state.clone();
        let local_motion_tx = local_sender.clone();
        tokio::spawn(async move {
            log::debug!("Accepted connection from: {}", addr);

            let mut client = glonax::transport::Client::new(stream);

            // TODO: Handle errors
            // TODO: Set timeout
            let start = client
                .recv_start()
                .await
                .expect("Failed to receive start message");

            let mut session_shutdown = false;

            log::info!("Session started for: {}", start.name());

            while let Ok(frame) = client.read_frame().await {
                match frame.message {
                    FrameMessage::Request => {
                        let request = client.request(frame.payload_length).await.unwrap();
                        match request.message() {
                            FrameMessage::Status => {
                                let status = &local_machine_state.read().await.status;
                                client.send_status(status).await.unwrap();
                            }
                            FrameMessage::Instance => {
                                let instance = glonax::core::Instance::new(
                                    local_config.instance.id.clone(),
                                    local_config.instance.model.clone(),
                                    local_config.instance.name.clone(),
                                );
                                client.send_instance(&instance).await.unwrap();
                            }
                            _ => todo!(),
                        }
                    }
                    FrameMessage::Shutdown => {
                        log::debug!("Client requested shutdown");
                        session_shutdown = true;
                        break;
                    }
                    FrameMessage::Motion => {
                        if start.is_write() {
                            let motion = client.motion(frame.payload_length).await.unwrap();

                            if let Err(e) = local_motion_tx.send(motion).await {
                                log::error!("Failed to send motion: {}", e);
                                break;
                            }
                        }
                    }
                    _ => {}
                }
            }

            if !session_shutdown && start.is_write() && start.is_failsafe() {
                log::warn!("Enacting failsafe for: {}", start.name());

                if let Err(e) = local_motion_tx.send(glonax::core::Motion::StopAll).await {
                    log::error!("Failed to send motion: {}", e);
                }
            }

            log::info!("Session shutdown for: {}", start.name());

            drop(permit);
        });
    }
}
