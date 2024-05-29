use nalgebra::{Rotation3, UnitVector3};

pub use error::{DeviceError, ErrorKind, Result};
pub use governor::Governor;
pub use hardware::nmea::Nmea;
pub use net::bosch_ems::BoschEngineManagementSystem;
pub use net::encoder::KueblerEncoder;
pub use net::engine::EngineMessage;
pub use net::fuzzer::Fuzzer;
pub use net::hydraulic::HydraulicControlUnit;
pub use net::inclino::KueblerInclinometer;
pub use net::inspector::{J1939ApplicationInspector, J1939Message};
pub use net::vcu::VehicleControlUnit;
pub use net::vms::VehicleManagementSystem;
pub use net::volvo_ems::VolvoD7E;
pub use r#virtual::encoder::VirtualEncoder;

mod error;
mod governor;
mod hardware;
pub mod net;
mod r#virtual;

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
    ///
    /// # Arguments
    ///
    /// * `factor` - The conversion factor to apply to the encoder position.
    /// * `offset` - The offset to apply to the encoder position.
    /// * `invert` - Whether to invert the encoder position.
    /// * `axis` - The axis of rotation for the encoder position.
    ///
    /// # Returns
    ///
    /// A new `EncoderConverter` instance.
    pub fn new(factor: f32, offset: f32, invert: bool, axis: UnitVector3<f32>) -> Self {
        Self {
            factor,
            offset,
            invert,
            axis,
        }
    }

    /// Convert encoder position to rotation.
    ///
    /// # Arguments
    ///
    /// * `position` - The encoder position to convert.
    ///
    /// # Returns
    ///
    /// The rotation corresponding to the encoder position.
    pub fn to_rotation(&self, position: f32) -> Rotation3<f32> {
        let position =
            ((position / self.factor) - self.offset) * if self.invert { -1.0 } else { 1.0 };

        Rotation3::from_axis_angle(&self.axis, position)
    }

    // TODO: This is incomplete
    /// Convert rotation to encoder position.
    ///
    /// # Arguments
    ///
    /// * `rotation` - The rotation to convert.
    ///
    /// # Returns
    ///
    /// The encoder position corresponding to the rotation.
    pub fn from_rotation(&self, rotation: Rotation3<f32>) -> u32 {
        let position = (std::f32::consts::PI * 2.0) - rotation.angle();

        ((position + self.offset) * self.factor) as u32
    }
}
