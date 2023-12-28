use glonax::{
    protocol::{
        frame::{Echo, Request, Session},
        Stream,
    },
    runtime::MotionSender,
};

use crate::{config::ProxyConfig, state::SharedExcavatorState};

async fn spawn_network_session(
    // stream: tokio::net::TcpStream,
    stream: tokio::net::UnixStream,
    // addr: std::net::SocketAddr,
    addr: tokio::net::unix::SocketAddr,
    runtime_state: SharedExcavatorState,
    motion_sender: MotionSender,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    use glonax::protocol::Packetize;

    // log::debug!("Accepted client from: {}", addr);

    let mut client = Stream::new(stream);

    // Always start with an anonymous session
    // let mut session = Session::new(0, addr.to_string());
    let mut session = Session::new(0, String::new());

    let mut session_shutdown = false;

    while let Ok(frame) = client.read_frame().await {
        match frame.message {
            glonax::protocol::frame::Request::MESSAGE_TYPE => {
                let request = client
                    .recv_packet::<Request>(frame.payload_length)
                    .await
                    .unwrap();
                match request.message() {
                    glonax::core::Instance::MESSAGE_TYPE => {
                        client
                            .send_packet(&runtime_state.read().await.instance)
                            .await
                            .unwrap();
                    }
                    glonax::core::Status::MESSAGE_TYPE => {
                        client
                            .send_packet(&runtime_state.read().await.status)
                            .await
                            .unwrap();
                    }
                    glonax::core::Pose::MESSAGE_TYPE => {
                        client
                            .send_packet(&runtime_state.read().await.state.pose)
                            .await
                            .unwrap();
                    }
                    glonax::core::Host::MESSAGE_TYPE => {
                        client
                            .send_packet(&runtime_state.read().await.state.vms)
                            .await
                            .unwrap();
                    }
                    glonax::core::Gnss::MESSAGE_TYPE => {
                        client
                            .send_packet(&runtime_state.read().await.state.gnss)
                            .await
                            .unwrap();
                    }
                    glonax::core::Engine::MESSAGE_TYPE => {
                        client
                            .send_packet(&runtime_state.read().await.state.engine)
                            .await
                            .unwrap();
                    }
                    // TODO: In v3 respond with error
                    _ => {}
                }
            }
            glonax::protocol::frame::Session::MESSAGE_TYPE => {
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
            glonax::protocol::frame::Echo::MESSAGE_TYPE => {
                let echo = client
                    .recv_packet::<Echo>(frame.payload_length)
                    .await
                    .unwrap();
                client.send_packet(&echo).await.unwrap();
            }
            glonax::protocol::frame::Shutdown::MESSAGE_TYPE => {
                log::debug!("Client requested shutdown");

                use tokio::io::AsyncWriteExt;

                client.inner_mut().shutdown().await.ok();

                session_shutdown = true;
                break;
            }
            glonax::core::Motion::MESSAGE_TYPE => {
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

// pub(super) async fn tcp_listen(
//     config: ProxyConfig,
//     runtime_state: SharedExcavatorState,
//     motion_sender: MotionSender,
// ) {
//     use tokio::net::TcpListener;

//     let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
//         glonax::consts::NETWORK_MAX_CLIENTS,
//     ));

//     log::debug!("Listening on: {}", config.address);
//     let listener = TcpListener::bind(config.address.clone()).await.unwrap();

//     loop {
//         let (stream, addr) = listener.accept().await.unwrap();

//         let permit = match semaphore.clone().try_acquire_owned() {
//             Ok(permit) => permit,
//             Err(_) => {
//                 log::warn!("Too many connections");
//                 continue;
//             }
//         };

//         let active_client_count =
//             glonax::consts::NETWORK_MAX_CLIENTS - semaphore.available_permits();

//         log::trace!(
//             "Connections: {}/{}",
//             active_client_count,
//             glonax::consts::NETWORK_MAX_CLIENTS
//         );

//         tokio::spawn(spawn_network_session(
//             stream,
//             addr,
//             runtime_state.clone(),
//             motion_sender.clone(),
//             permit,
//         ));
//     }
// }

pub(super) async fn unix_listen(
    config: ProxyConfig,
    runtime_state: SharedExcavatorState,
    motion_sender: MotionSender,
) {
    use tokio::net::UnixListener;

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
        glonax::consts::NETWORK_MAX_CLIENTS,
    ));

    // Remove the socket if it already exists
    if std::path::Path::new(glonax::consts::DEFAULT_SOCKET_PATH).exists() {
        std::fs::remove_file(glonax::consts::DEFAULT_SOCKET_PATH).unwrap();
    }

    log::debug!("Listening on: {}", glonax::consts::DEFAULT_SOCKET_PATH);
    let listener = UnixListener::bind(glonax::consts::DEFAULT_SOCKET_PATH).unwrap();

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
