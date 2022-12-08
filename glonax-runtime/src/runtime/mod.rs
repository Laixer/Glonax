mod motion;
pub(super) mod operand; // TODO: Why public
pub(super) mod program; // TODO: Why public

mod error;
use std::time::Duration;

use crate::signal;

pub use self::error::Error;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

mod builder;
pub(crate) use self::builder::Builder;
use self::operand::Operand;

pub mod ecu;
pub mod exec;
pub mod input;

#[async_trait::async_trait]
pub trait QueueAdapter: Send + Sync {
    fn topic(&self) -> &str;

    fn qos(&self) -> rumqttc::QoS;

    async fn parse(&mut self, event: &rumqttc::Publish);
}

pub struct EventHub {
    /// Broker client interface.
    client: std::sync::Arc<rumqttc::AsyncClient>,
    /// Local MQTT eventloop.
    eventloop: rumqttc::EventLoop,
    /// List of subscribed adapters.
    adapters: Vec<Box<dyn QueueAdapter>>,
}

impl EventHub {
    pub fn new(config: &crate::GlobalConfig) -> Self {
        use rumqttc::{AsyncClient, MqttOptions};

        let mut mqttoptions =
            MqttOptions::new(&config.bin_name, &config.mqtt_host, config.mqtt_port);
        mqttoptions
            .set_keep_alive(Duration::from_secs(5))
            .set_connection_timeout(1);

        if let (Some(username), Some(password)) = (&config.mqtt_username, &config.mqtt_password) {
            mqttoptions.set_credentials(username, password);
        }

        let (client, eventloop) = AsyncClient::new(mqttoptions, 100);
        let client = std::sync::Arc::new(client);

        Self {
            client,
            eventloop,
            adapters: vec![],
        }
    }

    pub fn subscribe<T: QueueAdapter + 'static>(&mut self, adapter: T) {
        self.client
            .try_subscribe(adapter.topic(), adapter.qos())
            .unwrap();

        self.adapters.push(Box::new(adapter));
    }

    pub async fn next(&mut self) {
        loop {
            match self.eventloop.poll().await {
                Ok(event) => {
                    if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(event)) = event {
                        for adapter in self.adapters.iter_mut() {
                            if event.topic == adapter.topic() {
                                adapter.parse(&event).await;
                            }
                        }
                    }
                }
                Err(e) => warn!("{}", e),
            };
        }
    }
}

pub struct RuntimeContext<K> {
    /// Runtime operand.
    pub(super) operand: K,
    /// Runtime event bus.
    pub(super) shutdown: (
        tokio::sync::broadcast::Sender<()>,
        tokio::sync::broadcast::Receiver<()>,
    ),
    /// Event hub.
    pub(super) eventhub: EventHub,
}

impl<K> RuntimeContext<K> {
    pub(crate) async fn new_network_gateway(
        &self,
        interface: &str,
        signal_manager: &crate::signal::SignalManager,
    ) -> self::Result<crate::device::Gateway> {
        debug!("Bind to interface {}", interface);

        let gateway_device = crate::device::Gateway::new(interface, signal_manager)
            .map_err(|_| Error::CoreDeviceNotFound)?;

        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            gateway_device.wait_online(),
        )
        .await
        .map_err(|_| Error::NetworkTimeout)?;

        Ok(gateway_device)
    }

    pub(super) fn new_signal_manager(&self) -> signal::SignalManager {
        signal::SignalManager::new(self.eventhub.client.clone())
    }

    pub(super) fn new_program_manager(&self) -> program::ProgramManager {
        program::ProgramManager::new(self.eventhub.client.clone())
    }

    pub(super) fn new_motion_manager(&self) -> motion::MotionManager {
        motion::MotionManager::new(self.eventhub.client.clone(), true)
    }
}
