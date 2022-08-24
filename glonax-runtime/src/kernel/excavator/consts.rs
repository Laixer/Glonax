use std::ops::Range;

// TODO: take all lengths in mm.

/// Maximum empirical driving speed in meters per second.
pub const DRIVE_SPEED_MAX: f32 = 26.1 / 30.0;
/// Boom length in meters.
pub const BOOM_LENGTH: f32 = 6.0;
/// Arm length in meters.
pub const ARM_LENGTH: f32 = 2.97;

// TODO: Rename. This is not an height but an transformation.
/// Frame height in meters.
#[allow(dead_code)]
pub const FRAME_HEIGHT: f32 = 1.885;

#[allow(dead_code)]
pub const BOOM_ORIGIN_OFFSET: (f32, f32) = (-0.784, 0.420);

/// Arm encoder range.
pub const ARM_ENCODER_RANGE: Range<f32> = 246.0..511.0;
/// Arm angle range.
pub const ARM_ANGLE_RANGE: Range<f32> = 0.0..2.1;
/// Boom encoder range.
pub const BOOM_ENCODER_RANGE: Range<f32> = 523.0..667.0;
/// Boom angle range.
pub const BOOM_ANGLE_RANGE: Range<f32> = 0.0..1.178;

/// Frame dimensions in (L)x(W)x(H)
#[allow(dead_code)]
const FRAME_DIMENSIONS: (f32, f32, f32) = (3.88, 2.89, 1.91);
// TODO: track hight.
/// Track dimensions in (L)x(W)x(H)
#[allow(dead_code)]
const TRACK_DIMENSIONS: (f32, f32, f32) = (4.65, 0.9, 0.0);

#[allow(dead_code)]
const SERVICE_POSITION_A: (f32, f32) = (0.0, 0.0);
#[allow(dead_code)]
const SERVICE_POSITION_B: (f32, f32) = (0.0, 0.0);
#[allow(dead_code)]
const SERVICE_POSITION_C: (f32, f32) = (0.0, 0.0);
#[allow(dead_code)]
const SERVICE_POSITION_D: (f32, f32) = (0.0, 0.0);
