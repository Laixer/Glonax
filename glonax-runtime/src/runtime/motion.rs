use std::sync::Arc;

use super::QueueAdapter;

const TOPIC: &str = "area/hydraulic/command";

pub struct MotionManager {
    #[allow(dead_code)]
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

    pub fn adapter(&self, motion_device: crate::device::Hcu) -> MotionQueueAdapter {
        MotionQueueAdapter {
            motion_device,
            motion_enabled: self.motion_enabled,
        }
    }

    #[allow(dead_code)]
    pub(super) fn publisher(&self) -> MotionPublisher {
        MotionPublisher {
            client: self.client.clone(),
            motion_enabled: self.motion_enabled,
        }
    }
}

pub struct MotionQueueAdapter {
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

        // if let Ok(motion) = postcard::from_bytes::<crate::core::motion::Motion>(&event.payload) {
        //     if self.motion_enabled {
        //         self.motion_device.actuate(motion).await;
        //     }
        // }
        if let Ok(str_payload) = std::str::from_utf8(&event.payload) {
            if let Ok(motion) = serde_json::from_str::<crate::core::motion::Motion>(str_payload) {
                if self.motion_enabled {
                    self.motion_device.actuate(motion).await;
                }
            }
        }
    }
}

#[allow(dead_code)]
pub(super) struct MotionPublisher {
    client: Arc<rumqttc::AsyncClient>,
    /// Whether or not to enable the motion device.
    motion_enabled: bool,
}

#[allow(dead_code)]
impl MotionPublisher {
    pub async fn publish<T: crate::core::motion::ToMotion>(&self, motion: T) {
        let motion = motion.to_motion();

        if self.motion_enabled {
            if let Ok(payload) = serde_json::to_string(&motion) {
                // if let Ok(payload) = postcard::to_stdvec(&motion) {
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
