pub trait SignalSource {
    fn collect_signals(&self, signals: &mut Vec<crate::core::Signal>);
}
