use crate::{
    core::Instance,
    transport::frame::{Frame, FrameMessage},
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
