use std::f32::consts::PI;

pub use geometry::*;
pub use motion::*;

mod geometry;
mod motion;

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
