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
    pub fn update(&mut self, error: f32) -> f32 {
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
