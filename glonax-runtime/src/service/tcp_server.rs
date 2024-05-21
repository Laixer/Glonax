use std::sync::Arc;

use tokio::{net::TcpListener, sync::Semaphore};

use crate::{
    core::{Object, ObjectMessage},
    runtime::{CommandSender, IPCSender, Service, ServiceContext},
};

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
    listener: Option<TcpListener>,
    clients: Vec<tokio::task::JoinHandle<()>>,
}

impl TcpServer {
    async fn spawn_client_session<T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin>(
        stream: T,
        ipc_tx: IPCSender,
        command_tx: CommandSender,
        _permit: tokio::sync::OwnedSemaphorePermit,
    ) {
        use crate::protocol::{
            frame::{Echo, Request, Session},
            Packetize, Stream,
        };

        log::debug!("Client session started");

        let mut client = Stream::new(stream);
        let mut session = Session::new(0, String::new());

        let mut session_shutdown = false;

        // TODO: If possible, move to glonax-runtime
        // TODO: Handle all unwraps, most just need to be logged
        // TODO: If possble, read the payload before the match
        // TODO: Seek frame header before reading the frame
        loop {
            match client.read_frame().await {
                Ok(frame) => match frame.message {
                    crate::protocol::frame::Request::MESSAGE_TYPE => {
                        let request = client
                            .recv_packet::<Request>(frame.payload_length)
                            .await
                            .unwrap();

                        match request.message() {
                            crate::core::Instance::MESSAGE_TYPE => {
                                client.send_packet(crate::global::instance()).await.unwrap();
                            }
                            // TODO: Not available at the moment
                            // crate::core::Status::MESSAGE_TYPE => {
                            //     client
                            //         .send_packet(&runtime_state.read().await.status())
                            //         .await
                            //         .unwrap();
                            // }
                            // TODO: Not available at the moment
                            // crate::core::Host::MESSAGE_TYPE => {
                            //     client
                            //         .send_packet(&runtime_state.read().await.state.vms_signal)
                            //         .await
                            //         .unwrap();
                            // }
                            // TODO: Not available at the moment
                            // crate::core::Gnss::MESSAGE_TYPE => {
                            //     client
                            //         .send_packet(&runtime_state.read().await.state.gnss_signal)
                            //         .await
                            //         .unwrap();
                            // }
                            // TODO: Not available at the moment
                            // crate::core::Engine::MESSAGE_TYPE => {
                            //     client
                            //         .send_packet(&runtime_state.read().await.state.engine_signal)
                            //         .await
                            //         .unwrap();
                            // }
                            // TODO: Not available at the moment
                            // crate::world::Actor::MESSAGE_TYPE => {
                            //     if let Some(actor) = &runtime_state.read().await.state.actor {
                            //         client.send_packet(actor).await.unwrap();
                            //     }
                            // }
                            _ => {
                                log::warn!("Unknown request: {}", request.message());
                            }
                        }
                    }
                    crate::protocol::frame::Session::MESSAGE_TYPE => {
                        session = client
                            .recv_packet::<Session>(frame.payload_length)
                            .await
                            .unwrap();

                        log::info!("Session started for: {}", session.name());

                        client.send_packet(crate::global::instance()).await.unwrap();
                    }
                    crate::protocol::frame::Echo::MESSAGE_TYPE => {
                        let echo = client
                            .recv_packet::<Echo>(frame.payload_length)
                            .await
                            .unwrap();

                        client.send_packet(&echo).await.unwrap();
                    }
                    // TODO: Replace with TCP shutdown
                    crate::protocol::frame::Shutdown::MESSAGE_TYPE => {
                        log::debug!("Session shutdown requested for: {}", session.name());

                        use tokio::io::AsyncWriteExt;

                        if let Err(e) = client.inner_mut().shutdown().await {
                            log::error!("Failed to shutdown stream: {}", e);
                        }

                        session_shutdown = true;
                        break;
                    }
                    crate::core::Engine::MESSAGE_TYPE => {
                        let engine = client
                            .recv_packet::<crate::core::Engine>(frame.payload_length)
                            .await
                            .unwrap();

                        if session.is_control() {
                            log::debug!("Engine request RPM: {}", engine.rpm);

                            if let Err(e) =
                                ipc_tx.send(ObjectMessage::command(Object::Engine(engine)))
                            {
                                log::error!("Failed to send target: {}", e);
                            }
                        } else {
                            log::warn!("Session is not authorized to control the machine");
                        }
                    }
                    crate::core::Motion::MESSAGE_TYPE => {
                        let motion = client
                            .recv_packet::<crate::core::Motion>(frame.payload_length)
                            .await
                            .unwrap();

                        if session.is_command() {
                            if let Err(e) = command_tx
                                .send(crate::core::Object::Motion(motion.clone()))
                                .await
                            {
                                log::error!("Failed to queue command: {}", e);
                            }
                            if let Err(e) =
                                ipc_tx.send(ObjectMessage::command(Object::Motion(motion)))
                            {
                                log::error!("Failed to send motion: {}", e);
                            }
                        } else {
                            log::warn!("Session is not authorized to command the machine");
                        }
                    }
                    crate::core::Target::MESSAGE_TYPE => {
                        let target = client
                            .recv_packet::<crate::core::Target>(frame.payload_length)
                            .await
                            .unwrap();

                        if session.is_command() {
                            if let Err(e) =
                                ipc_tx.send(ObjectMessage::command(Object::Target(target)))
                            {
                                log::error!("Failed to send target: {}", e);
                            }
                        } else {
                            log::warn!("Session is not authorized to command the machine");
                        }
                    }
                    crate::core::Control::MESSAGE_TYPE => {
                        let control = client
                            .recv_packet::<crate::core::Control>(frame.payload_length)
                            .await
                            .unwrap();

                        if session.is_control() {
                            if let Err(e) =
                                ipc_tx.send(ObjectMessage::command(Object::Control(control)))
                            {
                                log::error!("Failed to send control: {}", e);
                            }
                        } else {
                            log::warn!("Session is not authorized to control the machine");
                        }
                    }
                    _ => {
                        log::debug!("Unknown message: {}", frame.message);
                    }
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        log::warn!("Session abandoned for: {}", session.name());
                        break;
                    } else if e.kind() == std::io::ErrorKind::ConnectionReset {
                        log::warn!("Session reset for: {}", session.name());
                        break;
                    } else {
                        log::warn!("Failed to read frame: {}", e);
                    }
                }
            }
        }

        if !session_shutdown && session.is_control() && session.is_failsafe() {
            log::warn!("Enacting failsafe for: {}", session.name());

            if let Err(e) = command_tx
                .send(crate::core::Object::Motion(crate::core::Motion::StopAll))
                .await
            {
                log::error!("Failed to send motion: {}", e);
            }
        }

        log::info!("Session shutdown for: {}", session.name());
    }
}

impl Service<TcpServerConfig> for TcpServer {
    fn new(config: TcpServerConfig) -> Self
    where
        Self: Sized,
    {
        let semaphore = Arc::new(Semaphore::new(config.max_connections));

        Self {
            config,
            semaphore,
            listener: None,
            clients: Vec::new(),
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::with_address("tcp_server", self.config.listen.clone())
    }

    async fn setup(&mut self) {
        log::debug!("Listening on: {}", self.config.listen);

        // FUTURE: This is a bit of a hack, but there is no obvious way to create async constructors
        self.listener = Some(TcpListener::bind(self.config.listen.clone()).await.unwrap());
    }

    async fn wait_io(&mut self, ipc_tx: IPCSender, command_tx: CommandSender) {
        let (stream, addr) = self.listener.as_ref().unwrap().accept().await.unwrap();
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

        log::debug!(
            "Active connections: {}/{}",
            active_client_count,
            self.config.max_connections
        );

        log::debug!("Spawning client session");

        self.clients.push(tokio::spawn(Self::spawn_client_session(
            stream, ipc_tx, command_tx, permit,
        )));
    }

    async fn teardown(&mut self) {
        let active_client_count = self.config.max_connections - self.semaphore.available_permits();

        log::debug!(
            "Waiting for {} connected clients to shutdown",
            active_client_count
        );

        // TODO: Inform clients of shutdown

        // for client in self.clients.drain(..) {
        //     if let Err(e) = client.await {
        //         log::error!("Client session failed: {}", e);
        //     }
        // }
    }
}
