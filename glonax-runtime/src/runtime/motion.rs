use std::sync::Arc;

use super::QueueAdapter;

const TOPIC: &str = "command/actuate";

pub struct MotionManager {
    client: Arc<rumqttc::AsyncClient>,
    /// Whether or not to enable the motion device.
    motion_enabled: bool,
}

impl MotionManager {
    pub(super) fn new(client: Arc<rumqttc::AsyncClient>, motion_enabled: bool) -> Self {
        if !motion_enabled {
            info!("Motion device is disabled: no motion commands will be issued");
        }

        Self {
            client,
            motion_enabled,
        }
    }

    pub(super) fn adapter(&self, motion_device: crate::device::Hcu) -> MotionQueueAdapter {
        MotionQueueAdapter {
            motion_device,
            motion_enabled: self.motion_enabled,
        }
    }

    pub(super) fn publisher(&self) -> MotionPublisher {
        MotionPublisher {
            client: self.client.clone(),
            motion_enabled: self.motion_enabled,
        }
    }
}

pub(super) struct MotionQueueAdapter {
    /// Motion device.
    motion_device: crate::device::Hcu,
    /// Whether or not to enable the motion device.
    motion_enabled: bool,
}

#[async_trait::async_trait]
impl QueueAdapter for MotionQueueAdapter {
    fn topic(&self) -> &str {
        self::TOPIC
    }

    fn qos(&self) -> rumqttc::QoS {
        rumqttc::QoS::AtLeastOnce
    }

    async fn parse(&mut self, event: &rumqttc::Publish) {
        use crate::device::MotionDevice;

        if let Ok(motion) = postcard::from_bytes::<crate::core::motion::Motion>(&event.payload) {
            if self.motion_enabled {
                self.motion_device.actuate(motion).await;
            }
        }
    }
}

pub(super) struct MotionPublisher {
    client: Arc<rumqttc::AsyncClient>,
    /// Whether or not to enable the motion device.
    motion_enabled: bool,
}

impl MotionPublisher {
    pub async fn publish<T: crate::core::motion::ToMotion>(&self, motion: T) {
        let motion = motion.to_motion();

        if self.motion_enabled {
            if let Ok(payload) = postcard::to_stdvec(&motion) {
                match self
                    .client
                    .publish(TOPIC, rumqttc::QoS::AtLeastOnce, false, payload)
                    .await
                {
                    Ok(_) => trace!("Published motion: {}", motion),
                    Err(_) => warn!("Failed to publish motion"),
                }
            }
        }
    }
}
