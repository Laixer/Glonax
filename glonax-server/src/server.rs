use glonax::runtime::{MotionSender, SharedOperandState};

use crate::config::ProxyConfig;

pub(super) async fn service(
    config: ProxyConfig,
    runtime_state: SharedOperandState,
    sender: MotionSender,
    _shutdown: tokio::sync::broadcast::Receiver<()>,
) {
    use glonax::transport::frame::FrameMessage;
    use tokio::net::TcpListener;

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
        glonax::consts::NETWORK_MAX_CLIENTS,
    ));

    log::debug!("Listening on: {}", config.address);
    let listener = TcpListener::bind(config.address.clone()).await.unwrap();

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

        let local_runtime_state = runtime_state.clone();
        let local_motion_tx = sender.clone();
        tokio::spawn(async move {
            log::debug!("Accepted client from: {}", addr);

            let mut client = glonax::transport::Client::new(stream);

            // TODO: Set timeout
            let frame = match client.read_frame().await {
                Ok(frame) => frame,
                Err(e) => {
                    use tokio::io::AsyncWriteExt;

                    log::warn!("Failed to read frame: {}", e);

                    client
                        .send_packet(&glonax::transport::frame::Shutdown)
                        .await
                        .ok();
                    client.inner_mut().shutdown().await.ok();

                    log::debug!("Client shutdown");

                    return;
                }
            };

            // TODO: Handle errors
            let start = if frame.message == FrameMessage::Start {
                match client
                    .packet::<glonax::transport::frame::Start>(frame.payload_length)
                    .await
                {
                    Ok(start) => start,
                    Err(e) => {
                        use tokio::io::AsyncWriteExt;

                        log::warn!("Failed to read frame: {}", e);

                        client
                            .send_packet(&glonax::transport::frame::Shutdown)
                            .await
                            .ok();
                        client.inner_mut().shutdown().await.ok();

                        log::debug!("Client shutdown");

                        return;
                    }
                }
            } else {
                use tokio::io::AsyncWriteExt;

                log::warn!("Client did not start session");

                client
                    .send_packet(&glonax::transport::frame::Shutdown)
                    .await
                    .ok();
                client.inner_mut().shutdown().await.ok();

                log::debug!("Client shutdown");

                return;
            };

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
                            FrameMessage::Shutdown => {
                                log::debug!("Client requested shutdown");

                                use tokio::io::AsyncWriteExt;

                                client.inner_mut().shutdown().await.ok();

                                session_shutdown = true;
                                break;
                            }
                            FrameMessage::Status => {
                                client
                                    .send_packet(&local_runtime_state.read().await.status)
                                    .await
                                    .unwrap();
                            }
                            FrameMessage::Instance => {
                                client
                                    .send_packet(&local_runtime_state.read().await.instance)
                                    .await
                                    .unwrap();
                            }
                            FrameMessage::Pose => {
                                client
                                    .send_packet(&local_runtime_state.read().await.state.pose)
                                    .await
                                    .unwrap();
                            }
                            FrameMessage::Engine => {
                                client
                                    .send_packet(&local_runtime_state.read().await.state.engine)
                                    .await
                                    .unwrap();
                            }
                            FrameMessage::VMS => {
                                client
                                    .send_packet(&local_runtime_state.read().await.state.vms)
                                    .await
                                    .unwrap();
                            }
                            FrameMessage::GNSS => {
                                client
                                    .send_packet(&local_runtime_state.read().await.state.gnss)
                                    .await
                                    .unwrap();
                            }
                            _ => {}
                        }
                    }
                    FrameMessage::Shutdown => {
                        log::debug!("Client requested shutdown");

                        use tokio::io::AsyncWriteExt;

                        client.inner_mut().shutdown().await.ok();

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
