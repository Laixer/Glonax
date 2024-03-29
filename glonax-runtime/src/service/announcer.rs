use std::net::UdpSocket;

use crate::runtime::{Service, SharedOperandState};

pub struct Announcer(UdpSocket);

impl<C> Service<C> for Announcer {
    fn new(_config: C) -> Self
    where
        Self: Sized,
    {
        Self(UdpSocket::bind("[::1]:0").unwrap())
    }

    fn ctx(&self) -> crate::runtime::ServiceContext {
        crate::runtime::ServiceContext::new("announcer", Some("[::1]:0"))
    }

    async fn tick(&mut self, runtime_state: SharedOperandState) {
        // let instance = instance.clone();
        // let payload = [instance.to_bytes(), status.to_bytes()].concat();

        log::trace!("Sending instance and status broadcast");

        let runtime_state = runtime_state.read().await;

        let status = runtime_state.status();
        let payload = status.to_bytes();

        self.0.send_to(&payload, "[ff02::1]:30050").unwrap();
    }
}
