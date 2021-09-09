mod conn;

#[macro_use]
extern crate log;

use tokio::net::{TcpListener, ToSocketAddrs};

use crate::conn::Connection;

pub struct NetworkService {
    listener: TcpListener,
}

impl NetworkService {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> NetworkService {
        let listener = TcpListener::bind(addr).await.unwrap();

        info!("Listening on {}", listener.local_addr().unwrap());

        NetworkService { listener }
    }

    /// Start the network service.
    pub async fn launch(&self) {
        info!("Accepting connections");

        loop {
            let (stream, peer_addr) = self.listener.accept().await.unwrap();

            info!("Incoming connection from {}", peer_addr);

            tokio::spawn(async move {
                Connection::new(stream).read_frame().await;
            });
        }
    }
}
