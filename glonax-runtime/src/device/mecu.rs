use glonax_j1939::{Frame, PGN};

use crate::{
    net::KueblerEncoderService,
    signal::SignalQueueWriter,
    transport::{signal::Metric, Signal},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Actuator {
    Boom = 0,
    Arm = 4,
    Bucket = 5,
    Slew = 1,
    LimpLeft = 3,
    LimpRight = 2,
}

impl From<Actuator> for u32 {
    fn from(value: Actuator) -> Self {
        value as u32
    }
}

pub struct Mecu {
    writer: SignalQueueWriter,
    frame_encoder: KueblerEncoderService,
    arm_encoder: KueblerEncoderService,
    boom_encoder: KueblerEncoderService,
    attachment_encoder: KueblerEncoderService,
}

impl Mecu {
    pub fn new(writer: SignalQueueWriter) -> Self {
        Self {
            writer,
            frame_encoder: KueblerEncoderService::new(0x6A),
            boom_encoder: KueblerEncoderService::new(0x6B),
            arm_encoder: KueblerEncoderService::new(0x6C),
            attachment_encoder: KueblerEncoderService::new(0x6D),
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

            self.writer.send(Signal::new(
                Actuator::Arm,
                Metric::Angle(self.arm_encoder.position() as f32 / 1000.0),
            ));
            self.writer.send(Signal::new(
                Actuator::Arm,
                Metric::Rpm(self.arm_encoder.speed() as i32),
            ));

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

            self.writer.send(Signal::new(
                Actuator::Boom,
                Metric::Angle(self.boom_encoder.position() as f32 / 1000.0),
            ));
            self.writer.send(Signal::new(
                Actuator::Boom,
                Metric::Rpm(self.boom_encoder.speed() as i32),
            ));

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

            self.writer.send(Signal::new(
                Actuator::Slew,
                Metric::Angle(self.frame_encoder.position() as f32 / 1000.0),
            ));
            self.writer.send(Signal::new(
                Actuator::Slew,
                Metric::Rpm(self.frame_encoder.speed() as i32),
            ));

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

            self.writer.send(Signal::new(
                Actuator::Bucket,
                Metric::Angle(self.attachment_encoder.position() as f32 / 1000.0),
            ));
            self.writer.send(Signal::new(
                Actuator::Bucket,
                Metric::Rpm(self.attachment_encoder.speed() as i32),
            ));

            true
        } else {
            false
        }
    }
}
