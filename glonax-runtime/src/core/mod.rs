// pub mod algorithm;
pub mod input;
pub mod metric;
pub mod motion;

use std::f32::consts::PI;

// TODO: Remove?
// pub use nalgebra;

/// Level trait.
pub trait Level {
    /// Return the value of self above the lower threshold.
    /// Otherwise return a default value.
    fn ramp(self, lower: Self) -> Self;
}

impl Level for i16 {
    fn ramp(self, lower: Self) -> Self {
        if self < lower && self > -lower {
            0
        } else {
            self
        }
    }
}

pub mod time {
    use std::time::{Duration, SystemTime};

    /// Return the current time as a duration.
    #[inline]
    pub fn now() -> Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
    }
}

// TODO: TMP
pub fn deg_to_rad(input: f32) -> f32 {
    input * (PI / 180.0)
}

// TODO: TMP
pub fn rad_to_deg(input: f32) -> f32 {
    input * (180.0 / PI)
}

pub trait Identity {
    /// Introduction message.
    ///
    /// Returns a string to introduce the object for the first time and
    /// should only be called once.
    fn intro() -> String;
}

pub trait Tracer {
    type Instance;

    /// Create tracer from path.
    fn from_path<P: AsRef<std::path::Path>>(path: P) -> Self;

    /// Construct new tracer instance. Data recorded to this
    /// writer will be filed under the provided instance name.
    fn instance(&self, name: &str) -> Self::Instance;
}

pub trait TraceWriter {
    /// Write the record to the tracer.
    ///
    /// This stocks the record as part of the tracers series. A record must be
    /// serializable so that it can be consumed by binary tracers and its types
    /// persist.
    fn write_record<T: serde::Serialize>(&mut self, record: T);
}

pub trait Trace<T: TraceWriter> {
    /// Record the state of the object. How the object implements this
    /// is unspecified and left to the implementation.
    fn record(&self, writer: &mut T, timestamp: std::time::Duration);
}
