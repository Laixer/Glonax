use rand::Rng;

pub struct VirtualEncoder {
    rng: rand::rngs::OsRng,
    position: u32,
    factor: i16,
    bounds: (i16, i16),
    multiturn: bool,
    invert: bool,
}

impl VirtualEncoder {
    pub fn new(factor: i16, bounds: (i16, i16), multiturn: bool, invert: bool) -> Self {
        Self {
            rng: rand::rngs::OsRng,
            position: bounds.0 as u32, // TODO: Remove, we dont keep track of position here
            factor,
            bounds,
            multiturn,
            invert,
        }
    }

    pub fn update_position(&mut self, velocity: i16, jitter: bool) -> u32 {
        let velocity_norm = velocity / self.factor;
        let velocity_norm = if self.invert {
            -velocity_norm
        } else {
            velocity_norm
        };

        if self.multiturn {
            let mut position = (self.position as i16 + velocity_norm) % self.bounds.1;
            if position < 0 {
                position += self.bounds.1;
            }
            self.position = position as u32;
        } else {
            let mut position =
                (self.position as i16 + velocity_norm).clamp(self.bounds.0, self.bounds.1);
            if position < 0 {
                position += self.bounds.1;
            }
            self.position = position as u32;
        }

        if jitter && self.position < self.bounds.1 as u32 && self.position > 0 {
            self.position + self.rng.gen_range(0..=1)
        } else {
            self.position
        }
    }

    // TODO: This method may not be part of the encoder
    pub fn position_from_angle(&self, angle: f32) -> u32 {
        let position = std::f32::consts::PI * 2.0 - angle;
        self.position((position * 1_000.0) as u32, 0)
    }

    // TODO: Add optional jitter
    pub fn position(&self, possie: u32, velocity: i16) -> u32 {
        let velocity_norm = velocity / self.factor;
        let velocity_norm = if self.invert {
            -velocity_norm
        } else {
            velocity_norm
        };

        if self.multiturn {
            let mut position = (possie as i16 + velocity_norm) % self.bounds.1;
            if position < 0 {
                position += self.bounds.1;
            }
            position as u32
        } else {
            let mut position = (possie as i16 + velocity_norm).clamp(self.bounds.0, self.bounds.1);
            if position < 0 {
                position += self.bounds.1;
            }
            position as u32
        }
    }
}
