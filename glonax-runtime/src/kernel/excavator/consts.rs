// TODO: take all lengths in mm.

/// Maximum empirical driving speed in meters per second.
pub const DRIVE_SPEED_MAX: f32 = 26.1 / 30.0;
/// Boom length in meters.
pub const BOOM_LENGTH: f32 = 6.0;
/// Arm length in meters.
pub const ARM_LENGTH: f32 = 2.97;
// TODO: Rename. This is not an height but an transformation.
/// Frame height in meters.
pub const FRAME_HEIGHT: f32 = 1.885;
/// Arm angle range.
#[allow(dead_code)]
pub const ARM_RANGE: std::ops::Range<f32> = -0.45..-2.47;

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
