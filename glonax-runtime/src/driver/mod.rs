use nalgebra::{Rotation3, UnitVector3};

pub use error::{DeviceError, ErrorKind, Result};
pub use hardware::nmea::Nmea;
pub use net::bosch_ems::BoschEngineManagementSystem;
pub use net::encoder::KueblerEncoder;
pub use net::engine::EngineMessage;
pub use net::fuzzer::Fuzzer;
pub use net::hydraulic::HydraulicControlUnit;
pub use net::inclino::KueblerInclinometer;
pub use net::inspector::{J1939ApplicationInspector, J1939Message};
pub use net::reqres::RequestResponder;
pub use net::vcu::VehicleControlUnit;
pub use net::volvo_ems::VolvoD7E;
pub use r#virtual::encoder::VirtualEncoder;
pub use r#virtual::hcu::VirtualHCU;

mod error;
mod hardware;
pub mod net;
mod r#virtual;

// TODO: Move this to a more appropriate location.
pub struct EncoderConverter {
    /// Encoder factor.
    factor: f32,
    /// Encoder offset.
    offset: f32,
    /// Invert encoder.
    invert: bool,
    /// Encoder axis.
    axis: UnitVector3<f32>,
}

impl EncoderConverter {
    /// Create a new encoder converter.
    pub fn new(factor: f32, offset: f32, invert: bool, axis: UnitVector3<f32>) -> Self {
        Self {
            factor,
            offset,
            invert,
            axis,
        }
    }

    /// Convert encoder position to rotation.
    pub fn to_rotation(&self, position: f32) -> Rotation3<f32> {
        let position =
            ((position / self.factor) - self.offset) * if self.invert { -1.0 } else { 1.0 };

        Rotation3::from_axis_angle(&self.axis, position)
    }

    // TODO: This is incomplete
    pub fn from_rotation(&self, rotation: Rotation3<f32>) -> u32 {
        let position = (std::f32::consts::PI * 2.0) - rotation.angle();

        ((position + self.offset) * self.factor) as u32
    }
}
