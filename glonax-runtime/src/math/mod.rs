use std::f32::consts::PI;

pub use geometry::*;
pub use lin::*;

mod geometry;
mod lin;

/// Calculate the shortest rotation between two points on a circle
///
/// # Arguments
///
/// * `distance` - The distance between the two points
///
/// # Returns
///
/// The shortest rotation between the two points
///
/// # Examples
///
/// ```
/// use glonax::math::shortest_rotation;
///
/// let distance = 45.0_f32.to_radians();
/// let rotation = shortest_rotation(distance);
///
/// assert!(rotation < 46.0_f32.to_radians());
/// ```
pub fn shortest_rotation(distance: f32) -> f32 {
    let dist_normal = (distance + (2.0 * PI)) % (2.0 * PI);

    if dist_normal > PI {
        dist_normal - (2.0 * PI)
    } else {
        dist_normal
    }
}

/// Calculate the angle of a triangle using the law of cosines
///
/// # Arguments
///
/// * `a` - The length of side a
/// * `b` - The length of side b
/// * `c` - The length of side c
///
/// # Returns
///
/// The angle of the triangle
///
/// # Examples
///
/// ```
/// use glonax::math::law_of_cosines;
///
/// let a = 3.0;
/// let b = 4.0;
/// let c = 5.0;
/// let angle = law_of_cosines(a, b, c);
///
/// assert_eq!(angle, 1.5707964);
/// ```
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
/// This function calculates a linear motion profile based on the given delta.
/// The profile is scaled to the given scale and offset. The profile is bounded by the
/// lower bound. The profile is inverted if the inverse flag is set.
///
/// If the delta is less than the lower bound, `None` is returned.
pub fn linear_motion(
    delta: f32,
    lower_bound: f32,
    offset: f32,
    scale: f32,
    inverse: bool,
) -> Option<i16> {
    if delta.abs() < lower_bound {
        return None;
    }

    let delta_normal =
        ((delta.abs() * scale).min(i16::MAX as f32 - offset) + offset).round() as i16;

    let value = if delta.is_sign_negative() {
        delta_normal
    } else {
        -delta_normal
    };

    if inverse {
        Some(-value)
    } else {
        Some(value)
    }
}

/// Linear interpolation.
///
/// This function calculates the linear interpolation between two values.
///
/// # Arguments
///
/// * `a` - The start value
/// * `b` - The end value
/// * `t` - The interpolation factor
///
/// # Returns
///
/// The interpolated value
///
/// # Examples
///
/// ```
/// use glonax::math::lerp;
///
/// let a = 0.0;
/// let b = 1.0;
/// let t = 0.5;
/// let value = lerp(a, b, t);
///
/// assert_eq!(value, 0.5);
/// ```
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortest_rotation() {
        assert!(shortest_rotation(45.0_f32.to_radians()) < 46.0_f32.to_radians());
        assert!(shortest_rotation(179.0_f32.to_radians()) < 180.0_f32.to_radians());
    }

    #[test]
    fn test_linear_motion() {
        assert_eq!(
            linear_motion(-43.659_f32.to_radians(), 0.01, 12_000.0, 15_000.0, false),
            Some(23_430),
        );
        assert_eq!(
            linear_motion(-28.455_f32.to_radians(), 0.005, 12_000.0, 15_000.0, true),
            Some(-19_450),
        );
    }
}
