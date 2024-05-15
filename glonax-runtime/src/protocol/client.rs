use std::time::Duration;

use tokio::net::{TcpStream, ToSocketAddrs};

use crate::protocol::{frame, Stream};

pub async fn connect(
    address: impl ToSocketAddrs,
    session_name: impl ToString,
) -> std::io::Result<(Stream<TcpStream>, crate::core::Instance)> {
    connect_with(address, session_name, false, false).await
}

pub async fn connect_with(
    address: impl ToSocketAddrs,
    session_name: impl ToString,
    control: bool,
    failsafe: bool,
) -> std::io::Result<(Stream<TcpStream>, crate::core::Instance)> {
    let mut flags = 0;

    if control {
        flags |= frame::Session::MODE_CONTROL;
    } else {
        flags &= !frame::Session::MODE_CONTROL;
    }

    if failsafe {
        flags |= frame::Session::MODE_FAILSAFE;
    } else {
        flags &= !frame::Session::MODE_FAILSAFE;
    }

    let stream = tokio::net::TcpStream::connect(address).await?;

    let sock_ref = socket2::SockRef::from(&stream);

    let mut keep_alive = socket2::TcpKeepalive::new();
    keep_alive = keep_alive.with_time(Duration::from_secs(2));
    keep_alive = keep_alive.with_interval(Duration::from_secs(2));

    sock_ref.set_tcp_keepalive(&keep_alive)?;
    sock_ref.set_nodelay(true)?;

    let mut client = Stream::new(stream);

    let instance = client.handshake(session_name, flags).await?;

    Ok((client, instance))
}
