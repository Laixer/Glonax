use std::time::{Duration, SystemTime};

use glonax_core::{metric::MetricValue, Trace, TraceWriter};

use crate::device::{DeviceDescriptor, MetricDevice};

#[derive(Debug, Clone)]
pub struct Signal {
    /// Signal source.
    pub source: u32,
    /// Timestamp when this signal was received.
    pub timestamp: SystemTime,
    /// Signal value.
    pub value: MetricValue,
    /// Optional last value.
    pub last: Option<std::rc::Rc<Signal>>,
}

#[derive(serde::Serialize)]
struct SignalTrace {
    /// Timestamp of the trace.
    timestamp: u128,
    /// Signal source.
    source: u32,
    /// Generic value 0.
    value0: f32,
    /// Generic value 1.
    value1: f32,
    /// Generic value 2.
    value2: f32,
}

impl<T: TraceWriter> Trace<T> for Signal {
    fn record(&self, writer: &mut T, timestamp: Duration) {
        match self.value {
            MetricValue::Temperature(scalar) => writer.write_record(SignalTrace {
                timestamp: timestamp.as_millis(),
                source: self.source,
                value0: scalar as f32,
                value1: 0.0,
                value2: 0.0,
            }),
            MetricValue::Acceleration(vector) => writer.write_record(SignalTrace {
                timestamp: timestamp.as_millis(),
                source: self.source,
                value0: vector.x,
                value1: vector.y,
                value2: vector.z,
            }),
        }
    }
}

pub trait Sink {
    fn distribute(&mut self, domain: Signal);
}

pub(super) struct Pipeline<'a, W> {
    source_list: &'a mut Vec<DeviceDescriptor<dyn MetricDevice + Send>>,
    cache: std::collections::HashMap<u32, Signal>,
    tracer_instance: &'a mut W,
    timeout: Duration,
}

unsafe impl<W> Send for Pipeline<'_, W> {}

impl<'a, W: TraceWriter> Pipeline<'a, W> {
    /// Construct a new pipeline.
    pub(super) fn new(
        source_list: &'a mut Vec<DeviceDescriptor<dyn MetricDevice + Send>>,
        tracer_instance: &'a mut W,
        timeout: Duration,
    ) -> Self {
        Self {
            source_list,
            cache: std::collections::HashMap::new(),
            tracer_instance,
            timeout,
        }
    }

    // FUTURE: lock all devices at the same time.
    pub(super) async fn push_all<T: Sink + ?Sized>(&mut self, sink: &mut T) {
        for metric_device in self.source_list.iter_mut() {
            // Set the timeout and wait for the operation to complete. if the
            // timeout is reached this read is cancelled and we poll the next device.
            match tokio::time::timeout(self.timeout, metric_device.lock().await.next()).await {
                Ok(Some((id, value))) => {
                    let mut signal = Signal {
                        source: id as u32,
                        timestamp: SystemTime::now(),
                        value,
                        last: None,
                    };

                    signal.record(self.tracer_instance, glonax_core::time::now());

                    trace!("Source {} ⇨ {}", signal.source, signal.value);

                    signal.last = self
                        .cache
                        .insert(signal.source, signal.clone())
                        .map_or(None, |last_domain| Some(std::rc::Rc::new(last_domain)));

                    sink.distribute(signal);
                }
                Ok(None) => {}
                Err(_) => warn!("Timeout occured while reading from metric device"),
            }
        }
    }
}
