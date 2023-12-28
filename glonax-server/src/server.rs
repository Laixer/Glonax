use glonax::{
    protocol::{
        frame::{Echo, FrameMessage, Session},
        Client,
    },
    runtime::MotionSender,
};

use crate::{config::ProxyConfig, state::SharedExcavatorState};

async fn spawn_network_session(
    stream: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
    runtime_state: SharedExcavatorState,
    motion_sender: MotionSender,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    log::debug!("Accepted client from: {}", addr);

    let mut client = Client::new(stream);

    // Always start with an anonymous session
    let mut session = Session::new(0, addr.to_string());

    let mut session_shutdown = false;

    while let Ok(frame) = client.read_frame().await {
        // TODO: This is a bug in the making...
        match FrameMessage::from_u8(frame.message).unwrap() {
            FrameMessage::Request => {
                let request = client
                    .recv_packet::<glonax::protocol::frame::Request>(frame.payload_length)
                    .await
                    .unwrap();
                match request.message() {
                    FrameMessage::Instance => {
                        client
                            .send_packet(&runtime_state.read().await.instance)
                            .await
                            .unwrap();
                    }
                    FrameMessage::Status => {
                        client
                            .send_packet(&runtime_state.read().await.status)
                            .await
                            .unwrap();
                    }
                    FrameMessage::Pose => {
                        client
                            .send_packet(&runtime_state.read().await.state.pose)
                            .await
                            .unwrap();
                    }
                    FrameMessage::VMS => {
                        client
                            .send_packet(&runtime_state.read().await.state.vms)
                            .await
                            .unwrap();
                    }
                    FrameMessage::GNSS => {
                        client
                            .send_packet(&runtime_state.read().await.state.gnss)
                            .await
                            .unwrap();
                    }
                    FrameMessage::Engine => {
                        client
                            .send_packet(&runtime_state.read().await.state.engine)
                            .await
                            .unwrap();
                    }
                    // TODO: In v3 respond with error
                    _ => {}
                }
            }
            FrameMessage::Session => {
                session = client
                    .recv_packet::<Session>(frame.payload_length)
                    .await
                    .unwrap();

                log::info!("Session started for: {}", session.name());

                client
                    .send_packet(&runtime_state.read().await.instance)
                    .await
                    .unwrap();
            }
            FrameMessage::Echo => {
                let echo = client
                    .recv_packet::<Echo>(frame.payload_length)
                    .await
                    .unwrap();
                client.send_packet(&echo).await.unwrap();
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
                    .recv_packet::<glonax::core::Motion>(frame.payload_length)
                    .await
                    .unwrap();

                if session.is_control() {
                    if let Err(e) = motion_sender.send(motion).await {
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

    if !session_shutdown && session.is_control() && session.is_failsafe() {
        log::warn!("Enacting failsafe for: {}", session.name());

        if let Err(e) = motion_sender.send(glonax::core::Motion::StopAll).await {
            log::error!("Failed to send motion: {}", e);
        }
    }

    log::info!("Session shutdown for: {}", session.name());
}

pub(super) async fn service(
    config: ProxyConfig,
    runtime_state: SharedExcavatorState,
    motion_sender: MotionSender,
) {
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

        tokio::spawn(spawn_network_session(
            stream,
            addr,
            runtime_state.clone(),
            motion_sender.clone(),
            permit,
        ));
    }
}
