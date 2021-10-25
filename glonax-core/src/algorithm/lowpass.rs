/// Simple exponential smoothing filter.
pub struct SimpleExpSmoothing {
    /// Smoothing factor.
    alpha: f32,
    /// Best estimate for time series.
    s_t: Option<f32>,
}

impl SimpleExpSmoothing {
    /// Construct the filter.
    pub fn new(alpha: f32) -> Self {
        Self { alpha, s_t: None }
    }

    /// Feed the next value to the filter, then return the best forecast estimate.
    pub fn fit(&mut self, value: f32) -> f32 {
        let s0 = (self.alpha * value) + ((1.0 - self.alpha) * self.s_t.unwrap_or(value));
        self.s_t = Some(s0);
        s0
    }
}
