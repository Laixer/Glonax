use std::time::Duration;

use tokio::net::{TcpStream, ToSocketAddrs};

use crate::protocol::{frame, Stream};

pub struct ClientBuilder<A: ToSocketAddrs> {
    address: A,
    session_name: String,
    control: bool,
    command: bool,
    failsafe: bool,
    stream: bool,
}

impl<A: ToSocketAddrs> ClientBuilder<A> {
    pub fn new(address: A, session_name: impl ToString) -> Self {
        Self {
            address,
            session_name: session_name.to_string(),
            control: false,
            command: false,
            failsafe: false,
            stream: false,
        }
    }

    pub fn control(mut self, control: bool) -> Self {
        self.control = control;
        self
    }

    pub fn command(mut self, common: bool) -> Self {
        self.command = common;
        self
    }

    pub fn failsafe(mut self, failsafe: bool) -> Self {
        self.failsafe = failsafe;
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub async fn connect(self) -> std::io::Result<(Stream<TcpStream>, crate::core::Instance)> {
        let mut flags = 0;

        if self.control {
            flags |= frame::Session::MODE_CONTROL;
        } else {
            flags &= !frame::Session::MODE_CONTROL;
        }

        if self.command {
            flags |= frame::Session::MODE_COMMAND;
        } else {
            flags &= !frame::Session::MODE_COMMAND;
        }

        if self.failsafe {
            flags |= frame::Session::MODE_FAILSAFE;
        } else {
            flags &= !frame::Session::MODE_FAILSAFE;
        }

        if self.stream {
            flags |= frame::Session::MODE_STREAM;
        } else {
            flags &= !frame::Session::MODE_STREAM;
        }

        let stream = tokio::net::TcpStream::connect(self.address).await?;

        let sock_ref = socket2::SockRef::from(&stream);

        let mut keep_alive = socket2::TcpKeepalive::new();
        keep_alive = keep_alive.with_time(Duration::from_secs(2));
        keep_alive = keep_alive.with_interval(Duration::from_secs(2));

        sock_ref.set_tcp_keepalive(&keep_alive)?;
        sock_ref.set_nodelay(true)?;

        let mut client = Stream::new(stream);

        let instance = client.handshake(self.session_name, flags).await?;

        Ok((client, instance))
    }
}

pub async fn connect(
    address: impl ToSocketAddrs,
    session_name: impl ToString,
) -> std::io::Result<(Stream<TcpStream>, crate::core::Instance)> {
    ClientBuilder::new(address, session_name).connect().await
}
