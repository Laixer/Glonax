use std::f32::consts::PI;

pub use geometry::*;

mod geometry;

/// Calculate the shortest rotation between two points on a circle
pub fn shortest_rotation(distance: f32) -> f32 {
    let dist_normal = (distance + (2.0 * PI)) % (2.0 * PI);

    if dist_normal > PI {
        dist_normal - (2.0 * PI)
    } else {
        dist_normal
    }
}

/// Calculate the angle of a triangle using the law of cosines
pub fn law_of_cosines(a: f32, b: f32, c: f32) -> f32 {
    let a2 = a.powi(2);
    let b2 = b.powi(2);
    let c2 = c.powi(2);

    let numerator = a2 + b2 - c2;
    let denominator = 2.0 * a * b;

    (numerator / denominator).acos()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortest_rotation() {
        assert!(shortest_rotation(45.0_f32.to_radians()) < 46.0_f32.to_radians());
        assert!(shortest_rotation(179.0_f32.to_radians()) < 180.0_f32.to_radians());

        // TODO: More tests
    }
}
