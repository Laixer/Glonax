use glonax::{
    protocol::{
        frame::{Echo, Request, Session},
        Packetize, Stream,
    },
    runtime::{MotionSender, SharedOperandState},
};

async fn spawn_client_session<T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin>(
    stream: T,
    instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    motion_sender: MotionSender,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    let mut client = Stream::new(stream);

    // Always start with an anonymous session
    let mut session = Session::new(0, String::new());

    let mut session_shutdown = false;

    // TODO: If possible, move to glonax-runtime
    while let Ok(frame) = client.read_frame().await {
        match frame.message {
            glonax::protocol::frame::Request::MESSAGE_TYPE => {
                let request = client
                    .recv_packet::<Request>(frame.payload_length)
                    .await
                    .unwrap();

                // FUTURE: Pack into a single packet
                match request.message() {
                    glonax::core::Instance::MESSAGE_TYPE => {
                        client.send_packet(&instance).await.unwrap();
                    }
                    glonax::core::Status::MESSAGE_TYPE => {
                        client
                            .send_packet(&runtime_state.read().await.status())
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
                    glonax::world::Actor::MESSAGE_TYPE => {
                        if let Some(actor) = &runtime_state.read().await.state.actor {
                            client.send_packet(actor).await.unwrap();
                        }
                    }
                    // TODO: Respond with error
                    _ => {}
                }
            }
            glonax::protocol::frame::Session::MESSAGE_TYPE => {
                session = client
                    .recv_packet::<Session>(frame.payload_length)
                    .await
                    .unwrap();

                log::info!("Session started for: {}", session.name());

                client.send_packet(&instance).await.unwrap();
            }
            glonax::protocol::frame::Echo::MESSAGE_TYPE => {
                let echo = client
                    .recv_packet::<Echo>(frame.payload_length)
                    .await
                    .unwrap();

                client.send_packet(&echo).await.unwrap();
            }
            glonax::protocol::frame::Shutdown::MESSAGE_TYPE => {
                log::debug!("Client initiated shutdown");

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
                    // TODO: Move this further into the runtime
                    if motion.is_movable() {
                        if let Ok(mut runtime_state) = runtime_state.try_write() {
                            runtime_state.state.motion_instant = Some(std::time::Instant::now());
                        }
                    }

                    if let Err(e) = motion_sender.send(motion).await {
                        log::error!("Failed to queue motion: {}", e);
                        break;
                    }
                } else {
                    log::warn!("Client is not authorized to send motion");
                }
            }
            glonax::core::Target::MESSAGE_TYPE => {
                let target = client
                    .recv_packet::<glonax::core::Target>(frame.payload_length)
                    .await
                    .unwrap();

                if session.is_control() {
                    runtime_state.write().await.state.program.push_back(target);
                } else {
                    log::warn!("Client is not authorized to queue targets");
                }
            }
            glonax::core::Control::MESSAGE_TYPE => {
                let control = client
                    .recv_packet::<glonax::core::Control>(frame.payload_length)
                    .await
                    .unwrap();

                if session.is_control() {
                    match control {
                        glonax::core::Control::EngineRequest(rpm) => {
                            log::info!("Engine request RPM: {}", rpm);

                            let state = &mut runtime_state.write().await.state;
                            state.engine_state_request = Some(glonax::core::EngineRequest {
                                speed: rpm,
                                state: glonax::core::EngineState::Request,
                            });
                            state.engine_state_request_instant = Some(std::time::Instant::now());
                        }
                        glonax::core::Control::EngineShutdown => {
                            log::info!("Engine shutdown");

                            let state = &mut runtime_state.write().await.state;
                            state.engine_state_request = Some(glonax::core::EngineRequest {
                                speed: 0,
                                state: glonax::core::EngineState::NoRequest,
                            });
                            state.engine_state_request_instant = Some(std::time::Instant::now());
                        }

                        glonax::core::Control::HydraulicQuickDisconnect(on) => {
                            log::info!("Hydraulic quick disconnect: {}", on);
                            runtime_state.write().await.state.hydraulic_quick_disconnect = on;
                        }
                        glonax::core::Control::HydraulicLock(on) => {
                            log::info!("Hydraulic lock: {}", on);
                            runtime_state.write().await.state.hydraulic_lock = on;
                        }

                        glonax::core::Control::MachineShutdown => {
                            log::info!("Machine shutdown");
                        }
                        glonax::core::Control::MachineIllumination(on) => {
                            log::info!("Machine illumination: {}", on);
                        }
                        glonax::core::Control::MachineLights(on) => {
                            log::info!("Machine lights: {}", on);
                        }
                        glonax::core::Control::MachineHorn(on) => {
                            log::info!("Machine horn: {}", on);
                        }
                    }
                } else {
                    log::warn!("Client is not authorized to control the machine");
                }
            }
            _ => {
                log::debug!("Unknown message type: {}", frame.message);
            }
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

pub(super) async fn tcp_listen(
    config: crate::config::Config,
    instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    motion_sender: MotionSender,
) -> std::result::Result<(), glonax::runtime::ServiceError> {
    use glonax::runtime::ServiceErrorBuilder;
    use tokio::net::TcpListener;

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
        glonax::consts::NETWORK_MAX_CLIENTS,
    ));

    let tcp_server_config = config.tcp_server.clone().unwrap();

    let service = ServiceErrorBuilder::new("tcp_server", tcp_server_config.listen.clone());

    log::debug!("Listening on: {}", tcp_server_config.listen);
    let listener = TcpListener::bind(tcp_server_config.listen.clone())
        .await
        .map_err(|e| service.io_error(e))?;

    loop {
        let (stream, addr) = listener.accept().await.map_err(|e| service.io_error(e))?;
        stream.set_nodelay(true).map_err(|e| service.io_error(e))?;

        log::debug!("Accepted connection from: {}", addr);

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

        tokio::spawn(spawn_client_session(
            stream,
            instance.clone(),
            runtime_state.clone(),
            motion_sender.clone(),
            permit,
        ));
    }
}

pub(super) async fn unix_listen(
    config: crate::config::Config,
    instance: glonax::core::Instance,
    runtime_state: SharedOperandState,
    motion_sender: MotionSender,
) -> std::result::Result<(), glonax::runtime::ServiceError> {
    use glonax::runtime::ServiceErrorBuilder;
    use std::os::unix::fs::PermissionsExt;
    use tokio::net::UnixListener;

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
        glonax::consts::NETWORK_MAX_CLIENTS,
    ));

    let unix_server_config = config.unix_server.clone().unwrap();

    let path = unix_server_config.path.as_path();

    let service = ServiceErrorBuilder::new("unix_server", path.display());

    if path.exists() {
        std::fs::remove_file(path).map_err(|e| service.io_error(e))?;
    }

    log::debug!("Listening on: {}", path.display());
    let listener = UnixListener::bind(path).map_err(|e| service.io_error(e))?;

    tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o777))
        .await
        .map_err(|e| service.io_error(e))?;

    loop {
        let (stream, _addr) = listener.accept().await.map_err(|e| service.io_error(e))?;

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

        tokio::spawn(spawn_client_session(
            stream,
            instance.clone(),
            runtime_state.clone(),
            motion_sender.clone(),
            permit,
        ));
    }
}
