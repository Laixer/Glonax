use std::sync::Arc;

use glonax_j1939::{Frame, PGN};

use crate::{
    core::metric::{MetricValue, Signal},
    net::{J1939Network, KueblerEncoderService},
    signal::{Encoder, SignalPublisher},
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
            // trace!(
            //     "Arm Position: {}; Speed: {}",
            //     self.arm_encoder.position(),
            //     self.arm_encoder.speed()
            // );

            self.publisher.try_publish(
                "signal",
                Signal {
                    address: self.arm_encoder.node(),
                    subaddress: 0,
                    value: MetricValue::Angle(self.arm_encoder.position()),
                },
            );

            /// Arm encoder range.
            pub const ARM_ENCODER_RANGE: std::ops::Range<f32> = 25000.0..51800.0;
            /// Arm angle range.
            pub const ARM_ANGLE_RANGE: std::ops::Range<f32> = 0.0..2.1;

            let encoder = Encoder::new(ARM_ENCODER_RANGE, ARM_ANGLE_RANGE);

            let angle = encoder.scale(self.arm_encoder.position() as f32);
            let percentage = encoder.scale_to(100.0, self.arm_encoder.position() as f32);

            debug!(
                "Arm Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%",
                self.arm_encoder.position(),
                angle,
                crate::core::rad_to_deg(angle),
                percentage,
            );

            self.publisher
                .try_publish("body/arm", self.arm_encoder.position());
            self.publisher.try_publish(
                "body/arm2",
                EncoderSet {
                    position: self.arm_encoder.position(),
                    speed: self.arm_encoder.speed(),
                    angle,
                    percentage,
                },
            );

            true
        } else if self.boom_encoder.node() == frame.id().sa()
            && self.boom_encoder.ingress(pgn, frame)
        {
            // trace!(
            //     "Boom Position: {}; Speed: {}",
            //     self.boom_encoder.position(),
            //     self.boom_encoder.speed()
            // );

            self.publisher.try_publish(
                "signal",
                Signal {
                    address: self.boom_encoder.node(),
                    subaddress: 0,
                    value: MetricValue::Angle(self.boom_encoder.position()),
                },
            );

            /// Boom encoder range.
            pub const BOOM_ENCODER_RANGE: std::ops::Range<f32> = 136100.0..195600.0;
            /// Boom angle range.
            pub const BOOM_ANGLE_RANGE: std::ops::Range<f32> = 0.0..1.178;

            let encoder = Encoder::new(BOOM_ENCODER_RANGE, BOOM_ANGLE_RANGE);

            let angle = encoder.scale(self.boom_encoder.position() as f32);
            let percentage = encoder.scale_to(100.0, self.boom_encoder.position() as f32);

            debug!(
                "Boom Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%",
                self.boom_encoder.position(),
                angle,
                crate::core::rad_to_deg(angle),
                percentage,
            );

            self.publisher
                .try_publish("body/boom", self.boom_encoder.position());
            self.publisher.try_publish(
                "body/boom2",
                EncoderSet {
                    position: self.boom_encoder.position(),
                    speed: self.boom_encoder.speed(),
                    angle,
                    percentage,
                },
            );

            true
        } else if self.turn_encoder.node() == frame.id().sa()
            && self.turn_encoder.ingress(pgn, frame)
        {
            // trace!(
            //     "Turn Position: {}; Speed: {}",
            //     self.turn_encoder.position(),
            //     self.turn_encoder.speed()
            // );

            /// Slew encoder range.
            pub const SLEW_ENCODER_RANGE: std::ops::Range<f32> = 0.0..290000.0;
            /// Slew angle range.
            pub const SLEW_ANGLE_RANGE: std::ops::Range<f32> = 0.0..core::f32::consts::PI * 2.0;

            let encoder = Encoder::new(SLEW_ENCODER_RANGE, SLEW_ANGLE_RANGE);

            let angle = encoder.scale(self.turn_encoder.position() as f32);
            let percentage = encoder.scale_to(100.0, self.turn_encoder.position() as f32);

            debug!(
                "Turn Encoder: {:?}\tAngle rel.: {:>+5.2}rad {:>+5.2}° {:.1}%",
                self.turn_encoder.position(),
                angle,
                crate::core::rad_to_deg(angle),
                percentage,
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
            self.publisher.try_publish(
                "body/frame2",
                EncoderSet {
                    position: self.turn_encoder.position(),
                    speed: self.turn_encoder.speed(),
                    angle,
                    percentage,
                },
            );

            true
        } else {
            false
        }
    }
}
