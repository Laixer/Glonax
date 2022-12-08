use crate::core::metric::Signal;

mod encoder;
pub(crate) use encoder::Encoder;
use tokio::sync::mpsc;

const TOPIC: &str = "net/signal";

pub struct SignalManager {
    client: std::sync::Arc<rumqttc::AsyncClient>,
    queue: (
        mpsc::Sender<crate::core::metric::Signal>,
        mpsc::Receiver<crate::core::metric::Signal>,
    ),
}

impl SignalManager {
    /// Construct new signal manager.
    pub fn new(client: std::sync::Arc<rumqttc::AsyncClient>) -> Self {
        Self {
            client,
            queue: mpsc::channel(128),
        }
    }

    pub fn adapter(&self) -> SignalQueueAdapter {
        SignalQueueAdapter {
            queue: self.queue.0.clone(),
        }
    }

    pub fn publisher(&self) -> SignalPublisher {
        SignalPublisher {
            client: self.client.clone(),
        }
    }

    pub async fn recv(&mut self) -> Option<Signal> {
        self.queue.1.recv().await
    }
}

pub struct SignalQueueAdapter {
    queue: mpsc::Sender<crate::core::metric::Signal>,
}

#[async_trait::async_trait]
impl crate::runtime::QueueAdapter for SignalQueueAdapter {
    fn topic(&self) -> &str {
        self::TOPIC
    }

    async fn parse(&mut self, event: &rumqttc::Publish) {
        if let Ok(str_payload) = std::str::from_utf8(&event.payload) {
            if let Ok(signal) = serde_json::from_str::<crate::core::metric::Signal>(str_payload) {
                if let Err(_) = self.queue.try_send(signal) {
                    trace!("Signal queue reached maximum capacity");
                }
            }
        }
    }
}

pub struct SignalPublisher {
    client: std::sync::Arc<rumqttc::AsyncClient>,
}

impl SignalPublisher {
    pub async fn publish(&mut self, signal: Signal) {
        if let Ok(str_payload) = serde_json::to_string(&signal) {
            match self
                .client
                .publish(
                    TOPIC,
                    rumqttc::QoS::AtMostOnce,
                    false,
                    str_payload.as_bytes(),
                )
                .await
            {
                Ok(_) => trace!("Published signal: {}", signal),
                Err(_) => warn!("Failed to publish signal"),
            }
        }
    }
}
