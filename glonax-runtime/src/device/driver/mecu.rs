use std::sync::Arc;

use glonax_j1939::{Frame, PGN};

use crate::{
    core::metric::{MetricValue, Signal},
    net::{ControlNet, KueblerEncoderService},
    signal::SignalPublisher,
};

#[derive(Debug, serde::Serialize)]
struct EncoderData {
    position: u32,
    speed: u16,
}

pub struct Mecu {
    publisher: SignalPublisher,
    arm_encoder: KueblerEncoderService,
    boom_encoder: KueblerEncoderService,
    turn_encoder: KueblerEncoderService,
    arm_encoder_data: Option<EncoderData>,
    boom_encoder_data: Option<EncoderData>,
    turn_encoder_data: Option<EncoderData>,
}

impl Mecu {
    pub fn new(net: Arc<ControlNet>, publisher: SignalPublisher) -> Self {
        Self {
            publisher,
            arm_encoder: KueblerEncoderService::new(net.clone(), 0x6C),
            boom_encoder: KueblerEncoderService::new(net.clone(), 0x6A),
            turn_encoder: KueblerEncoderService::new(net.clone(), 0x20),
            arm_encoder_data: None,
            boom_encoder_data: None,
            turn_encoder_data: None,
        }
    }
}

#[async_trait::async_trait]
impl crate::net::Routable for Mecu {
    fn node(&self) -> u8 {
        0xff
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if self.arm_encoder.node() == frame.id().sa() && self.arm_encoder.ingress(pgn, frame) {
            self.arm_encoder_data = Some(EncoderData {
                position: self.arm_encoder.position(),
                speed: self.arm_encoder.speed(),
            });

            true
        } else if self.boom_encoder.node() == frame.id().sa()
            && self.boom_encoder.ingress(pgn, frame)
        {
            self.boom_encoder_data = Some(EncoderData {
                position: self.boom_encoder.position(),
                speed: self.boom_encoder.speed(),
            });

            true
        } else if self.turn_encoder.node() == frame.id().sa()
            && self.turn_encoder.ingress(pgn, frame)
        {
            self.turn_encoder_data = Some(EncoderData {
                position: self.turn_encoder.position(),
                speed: self.turn_encoder.speed(),
            });

            true
        } else {
            false
        }
    }

    async fn postroute(&mut self) {
        if let Some(data) = self.arm_encoder_data.take() {
            trace!("Arm Position: {}; Speed: {}", data.position, data.speed);

            self.publisher
                .publish(Signal {
                    address: self.arm_encoder.node(),
                    subaddress: 0,
                    value: MetricValue::Angle(data.position),
                })
                .await;
        }

        if let Some(data) = self.boom_encoder_data.take() {
            trace!("Boom Position: {}; Speed: {}", data.position, data.speed);

            self.publisher
                .publish(Signal {
                    address: self.boom_encoder.node(),
                    subaddress: 0,
                    value: MetricValue::Angle(data.position),
                })
                .await;
        }

        if let Some(data) = self.turn_encoder_data.take() {
            trace!("Turn Position: {}; Speed: {}", data.position, data.speed);

            self.publisher
                .publish(Signal {
                    address: self.turn_encoder.node(),
                    subaddress: 0,
                    value: MetricValue::Angle(data.position),
                })
                .await;
        }
    }
}
