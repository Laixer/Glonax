#[derive(Debug)]
pub struct Stats {
    /// Total number of ingress frames including failures.
    pub rx_count: usize,
    /// Number of malformed ingress frames.
    pub rx_failure: usize,
    /// Total number egress frames.
    pub tx_count: usize,
    /// Number of malformed egress frames.
    pub tx_failure: usize,
}

impl Stats {
    /// Create new empty statistics.
    pub fn new() -> Self {
        Self {
            rx_count: 0,
            rx_failure: 0,
            tx_count: 0,
            tx_failure: 0,
        }
    }

    /// Calculate ingress faillure rate in percentage.
    pub fn rx_faillure_rate(&self) -> usize {
        if self.rx_count > 0 {
            (self.rx_failure / self.rx_count) * 100
        } else {
            0
        }
    }

    /// Calculate egress faillure rate in percentage.
    pub fn tx_faillure_rate(&self) -> usize {
        if self.tx_count > 0 {
            (self.tx_failure / self.tx_count) * 100
        } else {
            0
        }
    }

    /// Reset statistics.
    pub fn reset(&mut self) {
        *self = Self::new()
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}
