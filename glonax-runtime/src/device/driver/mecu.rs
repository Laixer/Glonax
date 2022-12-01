use std::sync::Arc;

use glonax_j1939::{Frame, PGN};

use crate::{
    core::metric::{MetricValue, Signal},
    device::Device,
    net::ControlNet,
};

const DEVICE_NAME: &str = "m-ecu";

pub struct Mecu {
    pusher: crate::signal::SignalPusher,
}

impl Mecu {
    pub fn new(pusher: crate::signal::SignalPusher) -> Self {
        Self { pusher }
    }

    fn map_source(address: u8, subaddress: u8) -> u32 {
        ((address as u32) << 4) + subaddress as u32
    }
}

unsafe impl Send for Mecu {}

impl Device for Mecu {
    fn name(&self) -> String {
        DEVICE_NAME.to_owned()
    }
}

#[async_trait::async_trait]
impl super::gateway::GatewayClient for Mecu {
    fn from_net(_net: Arc<ControlNet>) -> Self {
        todo!()
    }

    async fn incoming(&mut self, frame: &Frame) {
        if frame.id().pgn() == PGN::ProprietaryB(65_535) {
            if frame.pdu()[..2] != [0xff, 0xff] {
                let data = u16::from_le_bytes(frame.pdu()[..2].try_into().unwrap());

                self.pusher
                    .push(
                        Self::map_source(frame.id().sa(), 0),
                        Signal::new(MetricValue::Stroke(nalgebra::Vector1::new(data))),
                    )
                    .await;
            }
            if frame.pdu()[2..4] != [0xff, 0xff] {
                let data = u16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap());

                self.pusher
                    .push(
                        Self::map_source(frame.id().sa(), 1),
                        Signal::new(MetricValue::Stroke(nalgebra::Vector1::new(data))),
                    )
                    .await;
            }
        } else if frame.id().pgn() == PGN::ProprietaryB(65_505) {
            if frame.pdu()[..6] != [0xff; 6] {
                let data_x = i16::from_le_bytes(frame.pdu()[..2].try_into().unwrap());
                let data_y = i16::from_le_bytes(frame.pdu()[2..4].try_into().unwrap());
                let data_z = i16::from_le_bytes(frame.pdu()[4..6].try_into().unwrap());

                self.pusher
                    .push(
                        Self::map_source(frame.id().sa(), 0),
                        Signal::new(MetricValue::Acceleration(nalgebra::Vector3::new(
                            data_x as f32,
                            data_y as f32,
                            data_z as f32,
                        ))),
                    )
                    .await;
            }
        } else if frame.id().pgn() == PGN::Other(64_258) {
            // TODO: Value may not be a u32
            let data = u32::from_le_bytes(frame.pdu()[..4].try_into().unwrap());

            self.pusher
                .push(
                    Self::map_source(frame.id().sa(), 0),
                    Signal::new(MetricValue::Angle(nalgebra::Vector1::new(
                        data.try_into().unwrap(),
                    ))),
                )
                .await;
        } else if frame.id().pgn() == PGN::Other(64_252) {
            let data = frame.pdu()[0];

            self.pusher
                .push(
                    Self::map_source(frame.id().sa(), 0),
                    Signal::new(MetricValue::Angle(nalgebra::Vector1::new(
                        data.try_into().unwrap(),
                    ))),
                )
                .await;
        } else if frame.id().pgn() == PGN::ProprietaryB(65_450) {
            let data = u32::from_le_bytes(frame.pdu()[0..4].try_into().unwrap());

            let data = (data / 100) as u16;

            self.pusher
                .push(
                    Self::map_source(frame.id().sa(), 0),
                    Signal::new(MetricValue::Angle(nalgebra::Vector1::new(
                        data.try_into().unwrap(),
                    ))),
                )
                .await;
        }
    }
}
