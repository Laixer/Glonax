pub(crate) struct Scalar(std::ops::Range<f32>);

impl Scalar {
    pub fn new(range: std::ops::Range<f32>) -> Self {
        Self(range)
    }

    pub fn normalize(&self, value: f32) -> f32 {
        let domain_value = if value < self.0.start {
            self.0.start
        } else if value > self.0.end {
            self.0.end
        } else {
            value
        };

        domain_value - self.0.start
    }

    pub fn scale(&self, domain: f32, value: f32) -> f32 {
        let delta = domain / (self.0.end - self.0.start);
        value * delta
    }
}
