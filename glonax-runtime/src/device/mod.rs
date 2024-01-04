use nalgebra::{Rotation3, UnitVector3};

pub use error::{DeviceError, ErrorKind, Result};
pub use hardware::encoder::KueblerEncoder;
pub use hardware::nmea::Nmea;
pub use r#virtual::encoder::VirtualEncoder;
pub use r#virtual::hcu::VirtualHCU;

mod error;
mod hardware;
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
    pub fn new(factor: f32, offset: f32, invert: bool, axis: UnitVector3<f32>) -> Self {
        Self {
            factor,
            offset,
            invert,
            axis,
        }
    }

    pub fn to_rotation(&self, position: u32) -> Rotation3<f32> {
        let position =
            ((position as f32 / self.factor) - self.offset) * if self.invert { -1.0 } else { 1.0 };

        Rotation3::from_axis_angle(&self.axis, position)
    }

    // TODO: This is incomplete
    pub fn from_rotation(&self, rotation: Rotation3<f32>) -> u32 {
        let position = (std::f32::consts::PI * 2.0) - rotation.angle();

        ((position + self.offset) * self.factor) as u32
    }
}
