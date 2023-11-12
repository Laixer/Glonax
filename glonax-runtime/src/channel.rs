// TODO: Move into mod
#[inline]
pub async fn any_bind() -> std::io::Result<tokio::net::UdpSocket> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true).unwrap();
    Ok(socket)
}
