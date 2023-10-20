use crate::config::ProxyConfig;

pub type MotionSender = tokio::sync::mpsc::Sender<glonax::core::Motion>;
pub type SharedMachineState = std::sync::Arc<tokio::sync::RwLock<glonax::MachineState>>;

pub(super) async fn _service_core(
    _local_config: ProxyConfig,
    _local_machine_state: SharedMachineState,
) {
    // use glonax::core::Metric;
    // use glonax::transport::frame::{Frame, FrameMessage};
    // use std::time::Instant;

    // log::debug!("Starting core service");

    // loop {
    //     tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    // }

    // // let mut now = Instant::now();

    // let mut signal_gnss_timeout = Instant::now();
    // let mut signal_encoder_timeout = Instant::now();
    // let mut signal_engine_timeout = Instant::now();

    //     if signal_gnss_timeout.elapsed().as_secs() > 5 {
    //         log::warn!("GNSS timeout: no update in last 5 seconds");
    //         local_machine_state.write().await.status = glonax::core::Status::DegradedTimeoutGNSS;
    //         signal_gnss_timeout = Instant::now();
    //     } else if signal_encoder_timeout.elapsed().as_secs() > 1 {
    //         log::warn!("Encoder timeout: no update in last 1 second");
    //         local_machine_state.write().await.status = glonax::core::Status::DegradedTimeoutEncoder;
    //         signal_encoder_timeout = Instant::now();
    //     } else if signal_engine_timeout.elapsed().as_secs() > 5 {
    //         log::warn!("Engine timeout: no update in last 5 seconds");
    //         local_machine_state.write().await.status = glonax::core::Status::DegradedTimeoutEngine;
    //         signal_engine_timeout = Instant::now();
    //     } else {
    //         local_machine_state.write().await.status = glonax::core::Status::Healthy;
    //     }

    //     let payload = signal.to_bytes();

    //     let mut frame = Frame::new(FrameMessage::Signal, payload.len());
    //     frame.put(&payload[..]);

    //     if let Err(e) = socket.send_to(frame.as_ref(), broadcast_addr).await {
    //         log::error!("Failed to send signal: {}", e);
    //         break;
    //     }
    // }

    // log::debug!("Signal broadcast shutdown");
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

    log::debug!("Listening on: {}", local_config.address);
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

        let active_client_count =
            glonax::consts::NETWORK_MAX_CLIENTS - semaphore.available_permits();

        log::trace!(
            "Connections: {}/{}",
            active_client_count,
            glonax::consts::NETWORK_MAX_CLIENTS
        );

        let local_config = local_config.clone();
        let local_machine_state = local_machine_state.clone();
        let local_motion_tx = local_sender.clone();
        tokio::spawn(async move {
            log::debug!("Accepted client from: {}", addr);

            let mut client = glonax::transport::Client::new(stream);

            // TODO: Set timeout
            let frame = match client.read_frame().await {
                Ok(frame) => frame,
                Err(e) => {
                    log::warn!("Failed to read frame: {}", e);
                    return;
                }
            };

            // TODO: Handle errors
            let start = if frame.message == FrameMessage::Start {
                client
                    .packet::<glonax::transport::frame::Start>(frame.payload_length)
                    .await
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid start message",
                ))
            }
            .expect("Failed to receive start message");

            let mut session_shutdown = false;

            log::info!("Session started for: {}", start.name());

            while let Ok(frame) = client.read_frame().await {
                match frame.message {
                    FrameMessage::Request => {
                        let request = client
                            .packet::<glonax::transport::frame::Request>(frame.payload_length)
                            .await
                            .unwrap();
                        match request.message() {
                            FrameMessage::Null => {
                                client
                                    .send_packet(&glonax::transport::frame::Null)
                                    .await
                                    .unwrap();
                            }
                            FrameMessage::Status => {
                                let status = &local_machine_state.read().await.status;
                                client.send_packet(status).await.unwrap();
                            }
                            FrameMessage::Instance => {
                                // TODO: Get this from the runtime session.
                                let instance = glonax::core::Instance::new(
                                    local_config.instance.id.clone(),
                                    local_config.instance.model.clone(),
                                    local_config.instance.name.clone(),
                                );
                                client.send_packet(&instance).await.unwrap();
                            }
                            FrameMessage::Pose => {
                                client
                                    .send_packet(&local_machine_state.read().await.state.pose)
                                    .await
                                    .unwrap();
                            }
                            FrameMessage::Engine => {
                                client
                                    .send_packet(&local_machine_state.read().await.state.engine)
                                    .await
                                    .unwrap();
                            }
                            FrameMessage::VMS => {
                                client
                                    .send_packet(&local_machine_state.read().await.state.vms)
                                    .await
                                    .unwrap();
                            }
                            FrameMessage::GNSS => {
                                client
                                    .send_packet(&local_machine_state.read().await.state.gnss)
                                    .await
                                    .unwrap();
                            }
                            _ => {
                                client
                                    .send_packet(&glonax::transport::frame::Null)
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                    FrameMessage::Shutdown => {
                        log::debug!("Client requested shutdown");
                        session_shutdown = true;
                        break;
                    }
                    FrameMessage::Motion => {
                        let motion = client
                            .packet::<glonax::core::Motion>(frame.payload_length)
                            .await
                            .unwrap();

                        if start.is_write() {
                            if let Err(e) = local_motion_tx.send(motion).await {
                                log::error!("Failed to send motion: {}", e);
                                break;
                            }
                        } else {
                            log::warn!("Client is not authorized to send motion");
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
