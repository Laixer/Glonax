use std::{fs, path::PathBuf};

use crate::{
    core::{Control, Engine, Motion, Object, Target},
    runtime::{CommandSender, Service, ServiceContext, SignalReceiver},
};

const UNIX_SOCKET_PERMISSIONS: u32 = 0o660;

#[derive(Clone, Debug, serde_derive::Deserialize, PartialEq, Eq)]
pub struct UnixServerConfig {
    /// Unix domain socket path to listen on.
    #[serde(default = "UnixServerConfig::default_path")]
    pub path: PathBuf,
}

impl UnixServerConfig {
    fn default_path() -> PathBuf {
        PathBuf::from("/tmp/glonax.sock")
    }
}

impl Default for UnixServerConfig {
    fn default() -> Self {
        Self {
            path: Self::default_path(),
        }
    }
}

// TODO: Rename to something other than 'TCPError'
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
    config: UnixServerConfig,
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
        use crate::protocol::{frame::Session, Packetize};

        match frame.message {
            crate::protocol::frame::Session::MESSAGE_TYPE => {
                *session = client
                    .recv_packet::<Session>(frame.payload_length)
                    .await
                    .map_err(TcpError::Io)?;

                let mut flags = Vec::new();

                if session.is_stream() {
                    flags.push("stream")
                }
                if session.is_failsafe() {
                    flags.push("failsafe")
                }

                log::info!(
                    "Session upgrade for {} with {}",
                    session.name(),
                    flags.join(", ")
                );

                client
                    .send_packet(crate::global::instance())
                    .await
                    .map_err(TcpError::Io)?;
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
                                log::warn!("Failed to process frame: {}", e);
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

impl Service<UnixServerConfig> for UnixServer {
    fn new(config: UnixServerConfig) -> Self
    where
        Self: Sized,
    {
        use std::os::unix::fs::PermissionsExt;

        if config.path.exists() {
            fs::remove_file(&config.path).unwrap();
        }

        let listener = tokio::net::UnixListener::bind(&config.path).unwrap();

        let permissions = fs::Permissions::from_mode(UNIX_SOCKET_PERMISSIONS);
        fs::set_permissions(&config.path, permissions).unwrap();

        Self { config, listener }
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::with_address(
            "unix_server",
            self.config.path.to_string_lossy().to_string(),
        )
    }

    async fn wait_io_sub(&mut self, command_tx: CommandSender, signal_rx: SignalReceiver) {
        let (stream, _) = self.listener.accept().await.unwrap();

        tokio::spawn(Self::spawn_client_session(stream, command_tx, signal_rx));
    }
}
