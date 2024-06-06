use std::time::Duration;

use crate::runtime::{CommandSender, NullConfig, Service, ServiceContext, SignalReceiver};

pub struct Pilot {}

impl Service<NullConfig> for Pilot {
    fn new(_: NullConfig) -> Self
    where
        Self: Sized,
    {
        Self {}
    }

    fn ctx(&self) -> ServiceContext {
        ServiceContext::new("pilot")
    }

    async fn wait_io_sub(&mut self, _command_tx: CommandSender, _signal_rx: SignalReceiver) {
        // let (stream, addr) = self.listener.as_ref().unwrap().accept().await.unwrap();

        // let sock_ref = socket2::SockRef::from(&stream);

        // let mut keep_alive = socket2::TcpKeepalive::new();
        // keep_alive = keep_alive.with_time(Duration::from_secs(2));
        // keep_alive = keep_alive.with_interval(Duration::from_secs(2));

        // sock_ref.set_tcp_keepalive(&keep_alive).unwrap();
        // sock_ref.set_nodelay(true).unwrap();

        // log::debug!("Accepted connection from: {}", addr);

        // let permit = match self.semaphore.clone().try_acquire_owned() {
        //     Ok(permit) => permit,
        //     Err(_) => {
        //         log::warn!("Too many connections");
        //         return;
        //     }
        // };

        // let active_client_count = self.config.max_connections - self.semaphore.available_permits();

        // log::debug!(
        //     "Active connections: {}/{}",
        //     active_client_count,
        //     self.config.max_connections
        // );

        // log::debug!("Spawning client session");

        // self.clients.push(tokio::spawn(Self::spawn_client_session(
        //     stream, command_tx, permit, signal_rx,
        // )));
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
