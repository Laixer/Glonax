use crate::{
    core::Instance,
    transport::frame::{Frame, FrameMessage},
    transport::Client,
};

// TODO: Move to lib
pub trait SignalSource {
    fn collect_signals(&self, signals: &mut Vec<crate::core::Signal>);
}

pub async fn recv_instance() -> std::io::Result<(Instance, std::net::IpAddr)> {
    let broadcast_addr = std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::UNSPECIFIED,
        crate::constants::DEFAULT_NETWORK_PORT,
    );

    let socket = tokio::net::UdpSocket::bind(broadcast_addr).await?;

    let mut buffer = [0u8; 1024];

    log::debug!("Waiting for instance announcement");

    loop {
        let (size, socket_addr) = socket.recv_from(&mut buffer).await?;
        if let Ok(frame) = Frame::try_from(&buffer[..size]) {
            if frame.message == FrameMessage::Instance {
                let instance = Instance::try_from(&buffer[frame.payload_range()]).unwrap();

                log::info!("Instance announcement received: {}", instance);

                return Ok((instance, socket_addr.ip()));
            }
        }
    }
}

pub async fn signal_open_write() -> std::io::Result<Client<tokio::fs::File>> {
    use crate::constants::FIFO_SIGNAL_FILE;

    log::debug!("Waiting for FIFO connection: {}", FIFO_SIGNAL_FILE);

    let client = Client::open_write(FIFO_SIGNAL_FILE).await?;

    log::debug!("Connected to FIFO: {}", FIFO_SIGNAL_FILE);

    Ok(client)
}
