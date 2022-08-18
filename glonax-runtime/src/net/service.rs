use std::{
    io,
    sync::Arc,
    time::{Duration, Instant},
};

use glonax_j1939::Frame;

use super::ControlNet;

pub struct ControlService {
    net: Arc<ControlNet>,
    last_actuator_ival: Instant,
    actuator_set: Option<(u8, std::collections::HashMap<u8, i16>)>,
}

impl ControlService {
    // TODO: rename
    pub fn from_net(net: Arc<ControlNet>) -> Self {
        Self {
            net,
            last_actuator_ival: Instant::now(),
            actuator_set: None,
        }
    }

    pub async fn interval(&mut self) {
        if self.last_actuator_ival.elapsed() >= Duration::from_millis(100) {
            if let Some((node, actuators)) = &self.actuator_set {
                trace!("Send actuator keepalive");

                self.net
                    .actuator_control(node.clone(), actuators.clone())
                    .await;
            }
            self.last_actuator_ival = Instant::now();
        }
    }

    pub async fn accept(&mut self) -> io::Result<Frame> {
        loop {
            if let Ok(frame) =
                tokio::time::timeout(Duration::from_millis(100), self.net.accept()).await
            {
                break frame;
            };

            self.interval().await
        }
    }

    #[inline]
    pub async fn accept_raw(&self) -> io::Result<Frame> {
        self.net.accept().await
    }

    /// Return a reference to the underlaying control net.
    pub fn net(&self) -> &ControlNet {
        &self.net
    }

    pub async fn actuator_control(
        &mut self,
        node: u8,
        actuators: std::collections::HashMap<u8, i16>,
    ) {
        self.net
            .actuator_control(node.clone(), actuators.clone())
            .await;

        self.actuator_set = Some((node, actuators))
    }
}

pub struct StatusService {
    net: Arc<ControlNet>,
    last_interval: Instant,
}

impl StatusService {
    pub fn new(net: Arc<ControlNet>) -> Self {
        Self {
            net,
            last_interval: Instant::now(),
        }
    }

    pub async fn interval(&mut self) {
        if self.last_interval.elapsed() >= Duration::from_secs(1) {
            self.net.announce_status().await;

            trace!("Announce host on network");

            self.last_interval = Instant::now();
        }
    }
}

pub struct ActuatorService {
    net: Arc<ControlNet>,
    node: u8,
    last_interval: Instant,
    actuator_set: Option<std::collections::HashMap<u8, i16>>,
}

impl ActuatorService {
    pub fn new(net: Arc<ControlNet>, node: u8) -> Self {
        Self {
            net,
            node,
            last_interval: Instant::now(),
            actuator_set: None,
        }
    }

    pub async fn interval(&mut self) {
        if self.last_interval.elapsed() >= Duration::from_millis(50) {
            if let Some(actuators) = &self.actuator_set {
                for (actuator, value) in actuators {
                    trace!("Keepalive: Change actuator {} to value {}", actuator, value);
                }

                self.net
                    .actuator_control(self.node, actuators.clone())
                    .await;
            }
            self.last_interval = Instant::now();
        }
    }

    /// Return a reference to the underlaying control net.
    pub fn net(&self) -> &ControlNet {
        &self.net
    }

    pub async fn lock(&mut self) {
        self.actuator_set = None;
        self.net.set_motion_lock(self.node, true).await;

        trace!("Disable motion");
    }

    pub async fn unlock(&mut self) {
        self.actuator_set = None;
        self.net.set_motion_lock(self.node, false).await;

        trace!("Enable motion");
    }

    pub async fn actuator_stop(&mut self, actuators: Vec<u8>) {
        // TODO: Log after await
        for actuator in &actuators {
            trace!("Stop actuator {}", actuator);
        }

        self.net
            .actuator_control(
                self.node,
                actuators.into_iter().map(|k| (k as u8, 0)).collect(),
            )
            .await;
    }

    pub async fn actuator_control(&mut self, actuators: std::collections::HashMap<u8, i16>) {
        // TODO: Log after await
        for (actuator, value) in &actuators {
            trace!("Change actuator {} to value {}", actuator, value);
        }

        self.net
            .actuator_control(self.node, actuators.clone())
            .await;

        // self.actuator_set = Some(actuators)
    }
}
