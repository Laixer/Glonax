use std::sync::Arc;

use tokio::{net::TcpListener, sync::Semaphore};

use crate::{
    core::{Object, ObjectMessage},
    runtime::{CommandSender, IPCSender, Service, ServiceContext, SignalReceiver},
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

enum TcpError {
    Io(std::io::Error),
    UnauthorizedControl,
    UnauthorizedCommand,
    Queue(std::sync::mpsc::SendError<ObjectMessage>),
    Command(tokio::sync::mpsc::error::SendError<Object>),
    UnknownMessage(u8),
}

impl std::fmt::Debug for TcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TcpError::Io(e) => write!(f, "IO error: {}", e),
            TcpError::UnauthorizedControl => write!(f, "Unauthorized control"),
            TcpError::UnauthorizedCommand => write!(f, "Unauthorized command"),
            TcpError::Queue(e) => write!(f, "Queue error: {}", e),
            TcpError::Command(e) => write!(f, "Command error: {}", e),
            TcpError::UnknownMessage(m) => write!(f, "Unknown message: {}", m),
        }
    }
}

impl std::fmt::Display for TcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TcpError::Io(e) => write!(f, "IO error: {}", e),
            TcpError::UnauthorizedControl => write!(f, "Unauthorized control"),
            TcpError::UnauthorizedCommand => write!(f, "Unauthorized command"),
            TcpError::Queue(e) => write!(f, "Queue error: {}", e),
            TcpError::Command(e) => write!(f, "Command error: {}", e),
            TcpError::UnknownMessage(m) => write!(f, "Unknown message: {}", m),
        }
    }
}

pub struct TcpServer {
    config: TcpServerConfig,
    semaphore: Arc<Semaphore>,
    listener: Option<TcpListener>,
    clients: Vec<tokio::task::JoinHandle<()>>,
}

impl TcpServer {
    // TODO: This method is barely readable. Refactor it.
    async fn parse<T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin>(
        client: &mut crate::protocol::Stream<T>,
        frame: &crate::protocol::frame::Frame,
        ipc_tx: IPCSender,
        command_tx: CommandSender,
        session: &mut crate::protocol::frame::Session,
    ) -> Result<(), TcpError> {
        use crate::protocol::{
            frame::{Echo, Session},
            Packetize,
        };

        match frame.message {
            crate::protocol::frame::Session::MESSAGE_TYPE => {
                *session = client
                    .recv_packet::<Session>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                let mut flags = Vec::new();
                if session.is_control() {
                    flags.push("control")
                }
                if session.is_command() {
                    flags.push("command")
                }
                if session.is_stream() {
                    flags.push("stream")
                }
                if session.is_failsafe() {
                    flags.push("failsafe")
                }

                log::info!(
                    "Session started for {} with {}",
                    session.name(),
                    flags.join(", ")
                );

                client
                    .send_packet(crate::global::instance())
                    .await
                    .map_err(TcpError::Io)?;
            }
            crate::protocol::frame::Echo::MESSAGE_TYPE => {
                let echo = client
                    .recv_packet::<Echo>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                client.send_packet(&echo).await.map_err(TcpError::Io)?;
            }
            crate::core::Engine::MESSAGE_TYPE => {
                let engine = client
                    .recv_packet::<crate::core::Engine>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                if session.is_control() {
                    log::debug!("Engine request RPM: {}", engine.rpm);

                    ipc_tx
                        .send(ObjectMessage::command(Object::Engine(engine)))
                        .map_err(TcpError::Queue)?;
                } else {
                    return Err(TcpError::UnauthorizedControl);
                }
            }
            crate::core::Motion::MESSAGE_TYPE => {
                let motion = client
                    .recv_packet::<crate::core::Motion>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                if session.is_command() {
                    command_tx
                        .send(Object::Motion(motion.clone()))
                        .await
                        .map_err(TcpError::Command)?;

                    // ipc_tx
                    //     .send(ObjectMessage::command(Object::Motion(motion)))
                    //     .map_err(TcpError::Queue)?;
                } else {
                    return Err(TcpError::UnauthorizedCommand);
                }
            }
            crate::core::Target::MESSAGE_TYPE => {
                let target = client
                    .recv_packet::<crate::core::Target>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                if session.is_command() {
                    ipc_tx
                        .send(ObjectMessage::command(Object::Target(target)))
                        .map_err(TcpError::Queue)?;
                } else {
                    return Err(TcpError::UnauthorizedCommand);
                }
            }
            crate::core::Control::MESSAGE_TYPE => {
                let control = client
                    .recv_packet::<crate::core::Control>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                if session.is_control() {
                    command_tx
                        .send(Object::Control(control))
                        .await
                        .map_err(TcpError::Command)?;

                    // ipc_tx
                    //     .send(ObjectMessage::command(Object::Control(control)))
                    //     .map_err(TcpError::Queue)?;
                } else {
                    return Err(TcpError::UnauthorizedControl);
                }
            }
            _ => {
                return Err(TcpError::UnknownMessage(frame.message));
            }
        }

        Ok(())
    }

    async fn spawn_client_session<T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin>(
        stream: T,
        ipc_tx: IPCSender,
        command_tx: CommandSender,
        _permit: tokio::sync::OwnedSemaphorePermit,
        mut signal_rx: SignalReceiver,
    ) {
        use crate::protocol::{frame::Session, Stream};

        log::debug!("Client session started");

        let mut client = Stream::new(stream);
        let mut session = Session::new(0, String::new());

        loop {
            tokio::select! {
                signal = signal_rx.recv() => {
                    if let Ok(signal) = signal {

                        if session.is_stream() {
                            match signal {
                                Object::Engine(engine) => {
                                    client.send_packet(&engine).await.unwrap();
                                }
                                Object::GNSS(gnss) => {
                                    client.send_packet(&gnss).await.unwrap();
                                }
                                Object::Host(vms) => {
                                    client.send_packet(&vms).await.unwrap();
                                }
                                Object::Motion(motion) => {
                                    client.send_packet(&motion).await.unwrap();
                                }
                                Object::Encoder(_) => {
                                    // TODO
                                }
                                _ => {}
                            }
                        }

                    } else if let Err(tokio::sync::broadcast::error::RecvError::Closed) = signal {
                        log::warn!("Signal channel closed");
                        break;
                    }
                }
                frame_rs = client.read_frame() => {
                    match frame_rs {
                        Ok(frame) => {
                            if let Err(e) = Self::parse(&mut client, &frame, ipc_tx.clone(), command_tx.clone(), &mut session).await {
                                log::warn!("Failed to parse frame: {}", e);
                            }
                        },
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                                log::debug!("Session shutdown requested for: {}", session.name());

                                use tokio::io::AsyncWriteExt;

                                client.inner_mut().shutdown().await.ok();

                                break;
                            } else if [
                                std::io::ErrorKind::ConnectionReset,
                                std::io::ErrorKind::TimedOut,
                                std::io::ErrorKind::ConnectionAborted,
                            ]
                            .contains(&e.kind())
                            {
                                log::warn!("Session reset for: {}", session.name());

                                if session.is_failsafe() {
                                    log::warn!("Enacting failsafe for: {}", session.name());

                                    if let Err(e) = command_tx.send(crate::core::Object::Motion(crate::core::Motion::StopAll)).await
                                    {
                                        log::error!("Failed to send motion: {}", e);
                                    }
                                }

                                break;
                            } else {
                                log::warn!("Failed to read frame: {}", e);
                            }
                        }
                    }
                }
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

    async fn wait_io(
        &mut self,
        ipc_tx: IPCSender,
        command_tx: CommandSender,
        signal_rx: SignalReceiver,
    ) {
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
            stream, ipc_tx, command_tx, permit, signal_rx,
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
