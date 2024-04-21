use std::sync::Arc;

use tokio::{net::TcpListener, sync::Semaphore};

use crate::runtime::{Service, ServiceContext, SharedOperandState};

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

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct UnixServerConfig {
    /// Unix domain socket path to listen on.
    pub path: std::path::PathBuf,
    /// Maximum number of connections.
    #[serde(default = "UnixServerConfig::default_max_connections")]
    pub max_connections: usize,
}

impl UnixServerConfig {
    fn default_max_connections() -> usize {
        10
    }
}

pub struct TcpServer {
    config: TcpServerConfig,
    semaphore: Arc<Semaphore>,
    listener: TcpListener,
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

        log::info!("Client session started");

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
                        crate::core::Instance::MESSAGE_TYPE => {
                            // TODO: Return the actual instance
                            let instance = crate::core::Instance::new(
                                "f0c4e7f1-f4e1-42b8-b002-afcdf1c76d12",
                                "kaas",
                                crate::core::MachineType::Excavator,
                                (1, 2, 3),
                            );
                            client.send_packet(&instance).await.unwrap();
                        }
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
                crate::protocol::frame::Session::MESSAGE_TYPE => {
                    session = client
                        .recv_packet::<Session>(frame.payload_length)
                        .await
                        .unwrap();

                    log::info!("Session started for: {}", session.name());

                    // TODO: Return the actual instance
                    let instance = crate::core::Instance::new(
                        "f0c4e7f1-f4e1-42b8-b002-afcdf1c76d12",
                        "kaas",
                        crate::core::MachineType::Excavator,
                        (1, 2, 3),
                    );
                    client.send_packet(&instance).await.unwrap();
                }
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
                crate::core::Engine::MESSAGE_TYPE => {
                    let engine = client
                        .recv_packet::<crate::core::Engine>(frame.payload_length)
                        .await
                        .unwrap();

                    if session.is_control() {
                        let state = &mut runtime_state.write().await.state;
                        state.engine_state_request = Some(crate::core::EngineRequest {
                            speed: engine.rpm,
                            state: crate::core::EngineState::Request,
                        });
                        state.engine_state_request_instant = Some(std::time::Instant::now());

                        log::debug!("Engine request RPM: {}", engine.rpm);
                    } else {
                        log::warn!("Client is not authorized to send engine data");
                    }
                }
                crate::core::Motion::MESSAGE_TYPE => {
                    let motion = client
                        .recv_packet::<crate::core::Motion>(frame.payload_length)
                        .await
                        .unwrap();

                    if session.is_control() {
                        // TODO: Move this further into the runtime
                        if motion.is_movable() {
                            if let Ok(mut runtime_state) = runtime_state.try_write() {
                                runtime_state.state.motion_instant =
                                    Some(std::time::Instant::now());
                            }
                        }

                        // if let Err(e) = motion_sender.send(motion).await {
                        //     log::error!("Failed to queue motion: {}", e);
                        //     break;
                        // }
                    } else {
                        log::warn!("Client is not authorized to send motion");
                    }
                }
                crate::core::Target::MESSAGE_TYPE => {
                    let target = client
                        .recv_packet::<crate::core::Target>(frame.payload_length)
                        .await
                        .unwrap();

                    if session.is_control() {
                        runtime_state.write().await.state.program.push_back(target);
                    } else {
                        log::warn!("Client is not authorized to queue targets");
                    }
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
                        }
                        crate::core::Control::EngineShutdown => {
                            log::info!("Engine shutdown");
                        }
                        crate::core::Control::HydraulicQuickDisconnect(on) => {
                            log::info!("Hydraulic quick disconnect: {}", on);
                        }
                        crate::core::Control::HydraulicLock(on) => {
                            log::info!("Hydraulic lock: {}", on);
                        }
                        crate::core::Control::HydraulicBoost(on) => {
                            log::info!("Hydraulic boost: {}", on);
                        }
                        crate::core::Control::MachineShutdown => {
                            log::info!("Machine shutdown");
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
                        crate::core::Control::MachineStrobeLight(on) => {
                            log::info!("Machine strobe light: {}", on);
                        }
                        crate::core::Control::MachineTravelAlarm(on) => {
                            log::info!("Machine travel light: {}", on);
                        }
                    }
                }
                _ => {
                    log::debug!("Unknown message type: {}", frame.message);
                }
            }
        }

        if !session_shutdown && session.is_control() && session.is_failsafe() {
            log::warn!("Enacting failsafe for: {}", session.name());

            // if let Err(e) = motion_sender.send(crate::core::Motion::StopAll).await {
            //     log::error!("Failed to send motion: {}", e);
            // }
        }

        log::info!("Session shutdown for: {}", session.name());
    }
}

impl Service<TcpServerConfig> for TcpServer {
    fn new(config: TcpServerConfig) -> Self
    where
        Self: Sized,
    {
        log::debug!("Listening on: {}", config.listen);

        let semaphore = Arc::new(Semaphore::new(config.max_connections));

        let core_listener = std::net::TcpListener::bind(config.listen.clone()).unwrap();
        let listener = tokio::net::TcpListener::from_std(core_listener).unwrap();

        Self {
            config,
            semaphore,
            listener,
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::with_address("tcp_server", self.config.listen.clone())
    }

    async fn wait_io(&mut self, runtime_state: SharedOperandState) {
        let rs =
            tokio::time::timeout(std::time::Duration::from_secs(1), self.listener.accept()).await;

        let (stream, addr) = match rs {
            Ok(Ok((stream, addr))) => (stream, addr),
            Ok(Err(e)) => {
                log::error!("Failed to accept connection: {}", e);
                return;
            }
            Err(_) => {
                return;
            }
        };

        // let (stream, addr) = self.listener.accept().await.unwrap();
        stream.set_nodelay(true).unwrap();

        log::debug!("Accepted connection from: {}", addr);

        let permit = match self.semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                log::warn!("Too many connections");
                return;
            }
        };

        let active_client_count = self.config.max_connections - self.semaphore.available_permits();

        log::trace!(
            "Connections: {}/{}",
            active_client_count,
            self.config.max_connections
        );

        tokio::spawn(Self::spawn_client_session(
            stream,
            runtime_state.clone(),
            permit,
        ));
    }
}
