use std::ops::Range;

// TODO: take all lengths in mm.

/// Maximum empirical driving speed in meters per second.
pub const DRIVE_SPEED_MAX: f32 = 26.1 / 30.0;
/// Boom length in meters.
pub const BOOM_LENGTH: f32 = 6.0;
/// Arm length in meters.
pub const ARM_LENGTH: f32 = 2.97;
/// Bucket length in meters.
#[allow(dead_code)]
pub const BUCKET_LENGTH: f32 = 1.493;

// TODO: Rename. This is not an height but an transformation.
/// Frame height in meters.
#[allow(dead_code)]
pub const FRAME_HEIGHT: f32 = 1.885;

#[allow(dead_code)]
pub const BOOM_ORIGIN_OFFSET: (f32, f32) = (-0.784, 0.420);

/// Boom encoder range.
pub const BOOM_ENCODER_RANGE: Range<f32> = 790.0..1017.0;
/// Boom angle range.
pub const BOOM_ANGLE_RANGE: Range<f32> = 0.0..1.178;
/// Arm encoder range.
pub const ARM_ENCODER_RANGE: Range<f32> = 247.0..511.0;
/// Arm angle range.
pub const ARM_ANGLE_RANGE: Range<f32> = 0.0..2.1;
/// Bucket encoder range.
pub const BUCKET_ENCODER_RANGE: Range<f32> = 424.0..697.0;
/// Bucket angle range.
pub const BUCKET_ANGLE_RANGE: Range<f32> = 0.0..3.0;
/// Slew encoder range.
pub const SLEW_ENCODER_RANGE: Range<f32> = 0.0..2899.0;
/// Slew angle range.
pub const SLEW_ANGLE_RANGE: Range<f32> = 0.0..core::f32::consts::PI * 2.0;

/// Frame dimensions in (L)x(W)x(H)
#[allow(dead_code)]
pub const FRAME_DIMENSIONS: (f32, f32, f32) = (3.88, 2.89, 1.91);
// TODO: track hight.
/// Track dimensions in (L)x(W)x(H)
#[allow(dead_code)]
pub const TRACK_DIMENSIONS: (f32, f32, f32) = (4.65, 0.9, 0.0);

/// Place the bucket on the ground in front of the machine.
#[allow(dead_code)]
pub const SERVICE_POSITION_A: (f32, f32) = (6.29, -0.49);
/// Strech the boom and arm with the bucket on the ground.
#[allow(dead_code)]
pub const SERVICE_POSITION_B: (f32, f32) = (8.52, -0.830);
/// Fold the bucket and arm in front of the machine.
#[allow(dead_code)]
pub const SERVICE_POSITION_C: (f32, f32) = (3.14, -1.45);

pub(super) const MOTION_PROFILE_SLEW: super::body::MotionProfile = super::body::MotionProfile {
    scale: 10_000.0,
    offset: 10_000,
    limit: 20_000,
    lower_bound: 0.02,
};

pub(super) const MOTION_PROFILE_BOOM: super::body::MotionProfile = super::body::MotionProfile {
    scale: 15_000.0,
    offset: 12_000,
    limit: 20_000,
    lower_bound: 0.02,
};

pub(super) const MOTION_PROFILE_ARM: super::body::MotionProfile = super::body::MotionProfile {
    scale: 15_000.0,
    offset: 12_000,
    limit: 20_000,
    lower_bound: 0.02,
};
