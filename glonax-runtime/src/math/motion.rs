/// Calculate linear motion profile.
///
/// This function calculates a linear motion profile based on the given delta value.
/// The profile is scaled to the given scale and offset. The profile is bounded by the
/// given lower bound.
///
/// If the delta value is less than the lower bound, `None` is returned.
pub fn linear_motion(
    delta: f32,
    lower_bound: f32,
    offset: f32,
    scale: f32,
    inverse: bool,
) -> Option<i32> {
    if delta.abs() < lower_bound {
        return None;
    }

    let delta_normal =
        ((delta.abs() * scale).min(std::i16::MAX as f32 - offset) + offset).round() as i32;

    let value = if delta.is_sign_negative() {
        -delta_normal
    } else {
        delta_normal
    };

    if inverse {
        Some(-value)
    } else {
        Some(value)
    }
}
