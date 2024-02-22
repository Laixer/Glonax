use tokio::net::TcpListener;

use crate::runtime::{Service, SharedOperandState};

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct TcpServerConfig {
    /// Network address to listen on.
    #[serde(default = "TcpServerConfig::default_listen")]
    pub listen: String,
    /// Maximum number of connections.
    #[serde(default = "TcpServerConfig::default_max_connections")]
    pub max_connections: usize,
}

impl TcpServerConfig {
    fn default_listen() -> String {
        "127.0.0.1:30051".to_owned()
    }

    fn default_max_connections() -> usize {
        10
    }
}

pub struct TcpServer {
    tcp_listener: TcpListener,
}

impl TcpServer {
    async fn spawn_client_session<T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin>(
        stream: T,
        runtime_state: SharedOperandState,
        _permit: tokio::sync::OwnedSemaphorePermit,
    ) {
        use crate::protocol::{
            frame::{Echo, Request, Session},
            Packetize, Stream,
        };

        let mut client = Stream::new(stream);

        let mut session = Session::new(0, String::new());

        let mut session_shutdown = false;

        // TODO: If possible, move to glonax-runtime
        while let Ok(frame) = client.read_frame().await {
            match frame.message {
                crate::protocol::frame::Request::MESSAGE_TYPE => {
                    let request = client
                        .recv_packet::<Request>(frame.payload_length)
                        .await
                        .unwrap();
                    // FUTURE: Pack into a single packet
                    match request.message() {
                        // crate::core::Instance::MESSAGE_TYPE => {
                        //     client.send_packet(&instance).await.unwrap();
                        // }
                        crate::core::Status::MESSAGE_TYPE => {
                            client
                                .send_packet(&runtime_state.read().await.status())
                                .await
                                .unwrap();
                        }
                        crate::core::Host::MESSAGE_TYPE => {
                            client
                                .send_packet(&runtime_state.read().await.state.vms)
                                .await
                                .unwrap();
                        }
                        crate::core::Gnss::MESSAGE_TYPE => {
                            client
                                .send_packet(&runtime_state.read().await.state.gnss)
                                .await
                                .unwrap();
                        }
                        crate::core::Engine::MESSAGE_TYPE => {
                            client
                                .send_packet(&runtime_state.read().await.state.engine)
                                .await
                                .unwrap();
                        }
                        crate::world::Actor::MESSAGE_TYPE => {
                            if let Some(actor) = &runtime_state.read().await.state.actor {
                                client.send_packet(actor).await.unwrap();
                            }
                        }
                        // TODO: Respond with error
                        _ => {}
                    }
                }
                // crate::protocol::frame::Session::MESSAGE_TYPE => {
                //     session = client
                //         .recv_packet::<Session>(frame.payload_length)
                //         .await
                //         .unwrap();

                //     log::info!("Session started for: {}", session.name());

                //     client.send_packet(&instance).await.unwrap();
                // }
                crate::protocol::frame::Echo::MESSAGE_TYPE => {
                    let echo = client
                        .recv_packet::<Echo>(frame.payload_length)
                        .await
                        .unwrap();

                    client.send_packet(&echo).await.unwrap();
                }
                crate::protocol::frame::Shutdown::MESSAGE_TYPE => {
                    log::debug!("Client initiated shutdown");

                    use tokio::io::AsyncWriteExt;

                    client.inner_mut().shutdown().await.ok();

                    session_shutdown = true;
                    break;
                }
                // crate::core::Motion::MESSAGE_TYPE => {
                //     let motion = client
                //         .recv_packet::<crate::core::Motion>(frame.payload_length)
                //         .await
                //         .unwrap();

                //     if session.is_control() {
                //         if let Err(e) = motion_sender.send(motion).await {
                //             log::error!("Failed to send motion: {}", e);
                //             break;
                //         }
                //     } else {
                //         log::warn!("Client is not authorized to send motion");
                //     }
                // }
                crate::core::Target::MESSAGE_TYPE => {
                    let target = client
                        .recv_packet::<crate::core::Target>(frame.payload_length)
                        .await
                        .unwrap();

                    runtime_state.write().await.state.program.push_back(target);
                }
                crate::core::Control::MESSAGE_TYPE => {
                    let control = client
                        .recv_packet::<crate::core::Control>(frame.payload_length)
                        .await
                        .unwrap();

                    match control {
                        crate::core::Control::EngineRequest(rpm) => {
                            let rpm = rpm.clamp(0, 2100);

                            log::info!("Engine request RPM: {}", rpm);

                            runtime_state.write().await.state.engine_request = rpm;
                        }
                        crate::core::Control::EngineShutdown => {
                            log::info!("Engine shutdown");
                            runtime_state.write().await.state.engine_request = 0;
                        }
                        crate::core::Control::HydraulicQuickDisconnect(on) => {
                            log::info!("Hydraulic quick disconnect: {}", on);
                        }
                        crate::core::Control::HydraulicLock(on) => {
                            log::info!("Hydraulic lock: {}", on);
                        }
                        crate::core::Control::MachineShutdown => {
                            log::info!("Machine shutdown");
                            runtime_state.write().await.state.engine_request = 0;
                            // runtime_state.write().await.state.engine.shutdown();
                        }
                        crate::core::Control::MachineIllumination(on) => {
                            log::info!("Machine illumination: {}", on);
                        }
                        crate::core::Control::MachineLights(on) => {
                            log::info!("Machine lights: {}", on);
                        }
                        crate::core::Control::MachineHorn(on) => {
                            log::info!("Machine horn: {}", on);
                        }
                    }
                }
                _ => {}
            }
        }

        // if !session_shutdown && session.is_control() && session.is_failsafe() {
        //     log::warn!("Enacting failsafe for: {}", session.name());

        //     if let Err(e) = motion_sender.send(crate::core::Motion::StopAll).await {
        //         log::error!("Failed to send motion: {}", e);
        //     }
        // }

        log::info!("Session shutdown for: {}", session.name());
    }
}

impl Service<TcpServerConfig> for TcpServer {
    fn new(config: TcpServerConfig) -> Self
    where
        Self: Sized,
    {
        log::debug!("Listening on: {}", config.listen);

        let listener = std::net::TcpListener::bind(config.listen).unwrap();

        Self {
            tcp_listener: TcpListener::from_std(listener).unwrap(),
        }
    }

    async fn wait_io(&mut self, runtime_state: SharedOperandState) {
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(10));

        loop {
            let (stream, addr) = self.tcp_listener.accept().await.unwrap();
            stream.set_nodelay(true).unwrap();

            log::debug!("Accepted connection from: {}", addr);

            let permit = match semaphore.clone().try_acquire_owned() {
                Ok(permit) => permit,
                Err(_) => {
                    log::warn!("Too many connections");
                    continue;
                }
            };

            let active_client_count = 10 - semaphore.available_permits();

            log::trace!("Connections: {}/{}", active_client_count, 10);

            tokio::spawn(Self::spawn_client_session(
                stream,
                runtime_state.clone(),
                permit,
            ));
        }
    }
}
