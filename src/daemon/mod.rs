mod conn;

use tokio::net::{TcpListener, ToSocketAddrs};

use crate::daemon::conn::Connection;

pub struct Service {
    listener: TcpListener,
}

impl Service {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> Service {
        let listener = TcpListener::bind(addr).await.unwrap();

        info!("Listening on {}", listener.local_addr().unwrap());

        Service { listener }
    }

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
