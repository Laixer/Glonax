use std::sync::Arc;

use glonax_j1939::{Frame, PGN};

use crate::{
    net::{J1939Network, KueblerEncoderService},
    signal::SignalPublisher,
};

#[derive(Debug, serde::Serialize)]
struct EncoderSet {
    position: u32,
    speed: u16,
    angle: f32,
    percentage: f32,
}

pub struct Mecu {
    publisher: SignalPublisher,
    frame_encoder: KueblerEncoderService,
    arm_encoder: KueblerEncoderService,
    boom_encoder: KueblerEncoderService,
    attachment_encoder: KueblerEncoderService,
}

impl Mecu {
    pub fn new(net: Arc<J1939Network>, publisher: SignalPublisher) -> Self {
        Self {
            publisher,
            frame_encoder: KueblerEncoderService::new(net.clone(), 0x6A),
            boom_encoder: KueblerEncoderService::new(net.clone(), 0x6B),
            arm_encoder: KueblerEncoderService::new(net.clone(), 0x6C),
            attachment_encoder: KueblerEncoderService::new(net, 0x6D),
        }
    }
}

impl crate::net::Routable for Mecu {
    fn node(&self) -> u8 {
        0xff
    }

    fn ingress(&mut self, pgn: PGN, frame: &Frame) -> bool {
        if self.arm_encoder.node() == frame.id().sa() && self.arm_encoder.ingress(pgn, frame) {
            debug!(
                "Arm Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}°",
                self.arm_encoder.position(),
                self.arm_encoder.position() as f32 / 1000.0,
                crate::core::rad_to_deg(self.arm_encoder.position() as f32 / 1000.0),
            );

            self.publisher.try_publish(
                "body/arm",
                EncoderSet {
                    position: self.arm_encoder.position(),
                    speed: self.arm_encoder.speed(),
                    angle: self.arm_encoder.position() as f32 / 1000.0,
                    percentage: 0.0,
                },
            );

            true
        } else if self.boom_encoder.node() == frame.id().sa()
            && self.boom_encoder.ingress(pgn, frame)
        {
            debug!(
                "Boom Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}°",
                self.boom_encoder.position(),
                self.boom_encoder.position() as f32 / 1000.0,
                crate::core::rad_to_deg(self.boom_encoder.position() as f32 / 1000.0),
            );

            self.publisher.try_publish(
                "body/boom",
                EncoderSet {
                    position: self.boom_encoder.position(),
                    speed: self.boom_encoder.speed(),
                    angle: self.boom_encoder.position() as f32 / 1000.0,
                    percentage: 0.0,
                },
            );

            true
        } else if self.frame_encoder.node() == frame.id().sa()
            && self.frame_encoder.ingress(pgn, frame)
        {
            debug!(
                "Frame Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}°",
                self.frame_encoder.position(),
                self.frame_encoder.position() as f32 / 1000.0,
                crate::core::rad_to_deg(self.frame_encoder.position() as f32 / 1000.0),
            );

            self.publisher.try_publish(
                "body/frame",
                EncoderSet {
                    position: self.frame_encoder.position(),
                    speed: self.frame_encoder.speed(),
                    angle: self.frame_encoder.position() as f32 / 1000.0,
                    percentage: 0.0,
                },
            );

            true
        } else if self.attachment_encoder.node() == frame.id().sa()
            && self.attachment_encoder.ingress(pgn, frame)
        {
            debug!(
                "Attachment Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}°",
                self.attachment_encoder.position(),
                self.attachment_encoder.position() as f32 / 1000.0,
                crate::core::rad_to_deg(self.attachment_encoder.position() as f32 / 1000.0),
            );

            self.publisher.try_publish(
                "body/attachment",
                EncoderSet {
                    position: self.attachment_encoder.position(),
                    speed: self.attachment_encoder.speed(),
                    angle: self.attachment_encoder.position() as f32 / 1000.0,
                    percentage: 0.0,
                },
            );

            true
        } else {
            false
        }
    }
}
