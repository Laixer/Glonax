use std::sync::Arc;

use glonax_j1939::{Frame, PGN};

use crate::{
    core::metric::{MetricValue, Signal},
    net::{J1939Network, KueblerEncoderService},
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
}

impl Mecu {
    pub fn new(net: Arc<J1939Network>, publisher: SignalPublisher) -> Self {
        Self {
            publisher,
            arm_encoder: KueblerEncoderService::new(net.clone(), 0x6C),
            boom_encoder: KueblerEncoderService::new(net.clone(), 0x6A),
            turn_encoder: KueblerEncoderService::new(net, 0x20),
        }
    }
}

impl crate::net::Routable for Mecu {
    fn node(&self) -> u8 {
        0xff
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if self.arm_encoder.node() == frame.id().sa() && self.arm_encoder.ingress(pgn, frame) {
            trace!(
                "Arm Position: {}; Speed: {}",
                self.arm_encoder.position(),
                self.arm_encoder.speed()
            );

            self.publisher.try_publish(
                "signal",
                Signal {
                    address: self.arm_encoder.node(),
                    subaddress: 0,
                    value: MetricValue::Angle(self.arm_encoder.position()),
                },
            );

            self.publisher
                .try_publish("body/arm", self.arm_encoder.position());

            true
        } else if self.boom_encoder.node() == frame.id().sa()
            && self.boom_encoder.ingress(pgn, frame)
        {
            trace!(
                "Boom Position: {}; Speed: {}",
                self.boom_encoder.position(),
                self.boom_encoder.speed()
            );

            self.publisher.try_publish(
                "signal",
                Signal {
                    address: self.boom_encoder.node(),
                    subaddress: 0,
                    value: MetricValue::Angle(self.boom_encoder.position()),
                },
            );

            self.publisher
                .try_publish("body/boom", self.boom_encoder.position());

            true
        } else if self.turn_encoder.node() == frame.id().sa()
            && self.turn_encoder.ingress(pgn, frame)
        {
            trace!(
                "Turn Position: {}; Speed: {}",
                self.turn_encoder.position(),
                self.turn_encoder.speed()
            );

            self.publisher.try_publish(
                "signal",
                Signal {
                    address: self.turn_encoder.node(),
                    subaddress: 0,
                    value: MetricValue::Angle(self.turn_encoder.position()),
                },
            );

            self.publisher
                .try_publish("body/frame", self.turn_encoder.position());

            true
        } else {
            false
        }
    }
}
