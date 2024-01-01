struct Linear {
    /// Proportional gain
    kp: f32,
    /// Derivative gain
    offset: f32,
    /// Integral of error
    inverse: bool,
}

impl Linear {
    /// Constructor to create a new linear controller
    pub fn new(kp: f32, offset: f32, inverse: bool) -> Linear {
        Linear {
            kp,
            offset,
            inverse,
        }
    }

    /// Method to update the linear controller based on the current error
    pub fn update(&self, error: f32) -> f32 {
        let normal = (error.abs() * self.kp).min(std::i16::MAX as f32 - self.offset) + self.offset;

        let value = if error.is_sign_negative() {
            normal
        } else {
            -normal
        };

        if self.inverse {
            -value
        } else {
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_1() {
        let linear = Linear::new(15_000.0, 12_000.0, false);

        let tolerance = 0.01;
        assert!((linear.update(-43.659_f32.to_radians()) - 23_429.9).abs() < tolerance);
    }

    #[test]
    fn test_linear_2() {
        let linear = Linear::new(15_000.0, 12_000.0, true);

        print!("{}", linear.update(-28.455_f32.to_radians()));

        let tolerance = 0.01;
        assert!((linear.update(-28.455_f32.to_radians()) + 19_449.5).abs() < tolerance);
    }
}
