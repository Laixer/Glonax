use std::path::PathBuf;

use crate::{
    core::{Control, Engine, Motion, Object, Target},
    runtime::{CommandSender, Service, ServiceContext, SignalReceiver},
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
    pub path: PathBuf,
}

enum TcpError {
    Io(std::io::Error),
    UnknownMessage(u8),
}

impl std::fmt::Debug for TcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TcpError::Io(e) => write!(f, "IO error: {}", e),
            TcpError::UnknownMessage(m) => write!(f, "Unknown message: {}", m),
        }
    }
}

impl std::fmt::Display for TcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TcpError::Io(e) => write!(f, "IO error: {}", e),
            TcpError::UnknownMessage(m) => write!(f, "Unknown message: {}", m),
        }
    }
}

// TODO: Rename to Server
pub struct UnixServer {
    // config: TcpServerConfig,
    listener: tokio::net::UnixListener,
}

impl UnixServer {
    // TODO: This method is barely readable. Refactor it.
    async fn parse<T: tokio::io::AsyncWrite + tokio::io::AsyncRead + Unpin>(
        client: &mut crate::protocol::Stream<T>,
        frame: &crate::protocol::frame::Frame,
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
                // if session.is_control() {
                //     flags.push("control")
                // }
                // if session.is_command() {
                //     flags.push("command")
                // }
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
            // TODO: Maybe we dont need echo anymore
            crate::protocol::frame::Echo::MESSAGE_TYPE => {
                let echo = client
                    .recv_packet::<Echo>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                client.send_packet(&echo).await.map_err(TcpError::Io)?;
            }
            Engine::MESSAGE_TYPE => {
                let engine = client
                    .recv_packet::<Engine>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                if let Err(e) = command_tx.send(Object::Engine(engine)) {
                    log::error!("Failed to command engine: {}", e);
                } else {
                    log::debug!("Engine request RPM: {}", engine.rpm);
                }
            }
            Motion::MESSAGE_TYPE => {
                let motion = client
                    .recv_packet::<Motion>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                if let Err(e) = command_tx.send(Object::Motion(motion.clone())) {
                    log::error!("Failed to command motion: {}", e);
                }
            }
            Target::MESSAGE_TYPE => {
                let target = client
                    .recv_packet::<Target>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                if let Err(e) = command_tx.send(Object::Target(target)) {
                    log::error!("Failed to command target: {}", e);
                } else {
                    log::debug!("Target request: {}", target);
                }
            }
            Control::MESSAGE_TYPE => {
                let control = client
                    .recv_packet::<Control>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                if let Err(e) = command_tx.send(Object::Control(control)) {
                    log::error!("Failed to command control: {}", e);
                } else {
                    log::debug!("Control request: {}", control);
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
        command_tx: CommandSender,
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
                                    if let Err(e) = client.send_packet(&engine).await {
                                        error!("Failed to send engine: {}", e);
                                    }
                                }
                                Object::GNSS(gnss) => {
                                    if let Err(e) = client.send_packet(&gnss).await {
                                        error!("Failed to send GNSS: {}", e);
                                    }
                                }
                                Object::Motion(motion) => {
                                    if let Err(e) = client.send_packet(&motion).await {
                                        error!("Failed to send motion: {}", e);
                                    }
                                }
                                Object::Rotator(rotator) => {
                                    if let Err(e) = client.send_packet(&rotator).await {
                                        error!("Failed to send rotator: {}", e);
                                    }
                                }
                                Object::ModuleStatus(status) => {
                                    if let Err(e) = client.send_packet(&status).await {
                                        error!("Failed to send status: {}", e);
                                    }
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
                            if let Err(e) = Self::parse(&mut client, &frame, command_tx.clone(), &mut session).await {
                                log::warn!("Failed to parse frame: {}", e);
                            }
                        },
                        Err(e) => {
                            match e.kind() {
                                std::io::ErrorKind::UnexpectedEof => {
                                    use tokio::io::AsyncWriteExt;

                                    client.inner_mut().shutdown().await.ok();

                                    log::debug!("Session shutdown requested for: {}", session.name());
                                    break;
                                },
                                std::io::ErrorKind::ConnectionReset => {
                                    log::warn!("Session reset for: {}", session.name());
                                    break;
                                },
                                std::io::ErrorKind::TimedOut => {
                                    log::warn!("Session timeout for: {}", session.name());
                                    break;
                                },
                                std::io::ErrorKind::ConnectionAborted => {
                                    log::warn!("Session aborted for: {}", session.name());
                                    break;
                                },
                                _ => {
                                    log::warn!("Failed to read frame: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }

        if session.is_failsafe() {
            log::info!("Enacting failsafe for: {}", session.name());

            if let Err(e) = command_tx.send(Object::Motion(Motion::StopAll)) {
                log::error!("Failed to command failsafe: {}", e);
            }
        }

        log::info!("Session shutdown for: {}", session.name());
    }
}

impl Service<crate::runtime::NullConfig> for UnixServer {
    fn new(_config: crate::runtime::NullConfig) -> Self
    where
        Self: Sized,
    {
        // "/run/glonax/glonax.sock"

        let socket_path = std::path::Path::new("/tmp/glonax.sock"); // TODO: Get from config
        if socket_path.exists() {
            std::fs::remove_file(socket_path).unwrap();
        }

        let listener = tokio::net::UnixListener::bind(socket_path).unwrap();

        Self {
            // config,
            listener,
        }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::with_address("unix_server", "/tmp/glonax.sock")
    }

    async fn wait_io_sub(&mut self, command_tx: CommandSender, signal_rx: SignalReceiver) {
        let (stream, _) = self.listener.accept().await.unwrap();

        log::debug!("Accepted local connection");

        tokio::spawn(Self::spawn_client_session(stream, command_tx, signal_rx));
    }
}
