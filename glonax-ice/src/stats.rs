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
    pub fn rx_failure_rate(&self) -> f64 {
        if self.rx_count > 0 {
            (self.rx_failure as f64 / self.rx_count as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate egress faillure rate in percentage.
    pub fn tx_failure_rate(&self) -> f64 {
        if self.tx_count > 0 {
            (self.tx_failure as f64 / self.tx_count as f64) * 100.0
        } else {
            0.0
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_is_empty() {
        let mut stats = Stats::new();

        stats.rx_count += 1;
        stats.reset();

        assert_eq!(stats.rx_count, 0);
        assert_eq!(stats.rx_failure, 0);
        assert_eq!(stats.tx_count, 0);
        assert_eq!(stats.tx_failure, 0);
    }

    #[test]
    fn failure_rate() {
        let mut stats = Stats::new();

        stats.rx_count += 100;
        stats.rx_failure += 5;

        assert_eq!(stats.rx_failure_rate(), 5.0);

        stats.tx_count += 50;
        stats.tx_failure += 9;

        assert_eq!(stats.tx_failure_rate(), 18.0);
    }
}
