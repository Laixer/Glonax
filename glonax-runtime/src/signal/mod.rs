use std::sync::Arc;

use crate::core::metric::Signal;

mod encoder;
pub(crate) use encoder::Encoder;
use tokio::sync::watch;

const TOPIC: &str = "net/signal";

pub struct SignalManager {
    client: Arc<rumqttc::AsyncClient>,
    sender: Option<watch::Sender<Signal>>,
    receiver: watch::Receiver<Signal>,
}

impl SignalManager {
    /// Construct new signal manager.
    pub fn new(client: Arc<rumqttc::AsyncClient>) -> Self {
        let (tx, rx) = watch::channel(Signal {
            address: 0x0,
            subaddress: 0,
            value: crate::core::metric::MetricValue::Angle(0),
        });

        Self {
            client,
            sender: Some(tx),
            receiver: rx,
        }
    }

    pub fn adapter(&mut self) -> SignalQueueAdapter {
        SignalQueueAdapter {
            sender: self.sender.take().unwrap(),
        }
    }

    pub fn publisher(&self) -> SignalPublisher {
        SignalPublisher {
            client: self.client.clone(),
        }
    }

    pub async fn recv(&mut self) -> Signal {
        self.receiver.changed().await.unwrap();
        *self.receiver.borrow()
    }
}

pub struct SignalQueueAdapter {
    sender: watch::Sender<Signal>,
}

#[async_trait::async_trait]
impl crate::runtime::QueueAdapter for SignalQueueAdapter {
    fn topic(&self) -> &str {
        self::TOPIC
    }

    async fn parse(&mut self, event: &rumqttc::Publish) {
        if let Ok(str_payload) = std::str::from_utf8(&event.payload) {
            if let Ok(signal) = serde_json::from_str::<Signal>(str_payload) {
                self.sender.send(signal).unwrap();
            }
        }
    }
}

pub struct SignalPublisher {
    client: Arc<rumqttc::AsyncClient>,
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
