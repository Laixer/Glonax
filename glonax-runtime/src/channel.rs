use crate::{consts, transport::Client};

// TODO: Move into mod
#[inline]
pub async fn any_bind() -> std::io::Result<tokio::net::UdpSocket> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true).unwrap();
    Ok(socket)
}

pub async fn signal_open_write() -> std::io::Result<Client<tokio::fs::File>> {
    use consts::FIFO_SIGNAL_FILE;

    log::debug!("Waiting for FIFO connection: {}", FIFO_SIGNAL_FILE);

    let client = Client::open_write(FIFO_SIGNAL_FILE).await?;

    log::debug!("Connected to FIFO: {}", FIFO_SIGNAL_FILE);

    Ok(client)
}
