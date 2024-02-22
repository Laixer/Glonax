use std::net::UdpSocket;

use crate::runtime::{Service, SharedOperandState};

pub struct Announcer(UdpSocket);

impl<C> Service<C> for Announcer {
    fn new(_config: C) -> Self
    where
        Self: Sized,
    {
        log::debug!("Starting network announcer service");

        let socket = UdpSocket::bind("[::1]:0").unwrap();

        Self(socket)
    }

    fn tick(&mut self, runtime_state: SharedOperandState) {
        // let instance = instance.clone();
        // let payload = [instance.to_bytes(), status.to_bytes()].concat();

        log::trace!("Sending instance and status broadcast");

        let status = if let Ok(runtime_state) = runtime_state.try_read() {
            Some(runtime_state.status())
        } else {
            None
        };

        if let Some(status) = status {
            let payload = status.to_bytes();

            self.0.send_to(&payload, "[ff02::1]:30050").unwrap();
        }
    }
}
