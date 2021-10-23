use std::time::{Duration, SystemTime};

use glonax_core::metric::MetricValue;

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

pub trait Sink {
    fn distribute(&mut self, domain: Signal);
}

pub(super) struct PipelineBuilder<'a> {
    pub(super) source_list: &'a mut Vec<DeviceDescriptor<dyn MetricDevice + Send>>,
    pub(super) trace_path: Option<std::path::PathBuf>,
    pub(super) timeout: Duration,
}

impl<'a> PipelineBuilder<'a> {
    /// Build the pipeline from its properties.
    ///
    /// This call consumes the builder.
    pub(super) fn build(self) -> Pipeline<'a> {
        let trace_writer = self.trace_path.map(|path| {
            let mut wtr = csv::Writer::from_path(path).unwrap();

            wtr.write_record(&["timestamp", "source", "value_0", "value_1", "value_2"])
                .unwrap();

            wtr
        });

        Pipeline {
            source_list: self.source_list,
            cache: std::collections::HashMap::new(),
            trace_writer,
            timeout: self.timeout,
        }
    }
}

pub(super) struct Pipeline<'a> {
    source_list: &'a mut Vec<DeviceDescriptor<dyn MetricDevice + Send>>,
    cache: std::collections::HashMap<u32, Signal>,
    trace_writer: Option<csv::Writer<std::fs::File>>,
    timeout: Duration,
}

unsafe impl Send for Pipeline<'_> {}

impl<'a> Pipeline<'a> {
    // FUTURE: lock all devices at the same time.
    pub(super) async fn push_all<T: Sink + ?Sized>(&mut self, sink: &mut T) {
        for metric_device in self.source_list.iter_mut() {
            // Set the timeout and wait for the operation to complete. if the
            // timeout is reached this read is cancelled and we poll the next device.
            match tokio::time::timeout(self.timeout, metric_device.lock().await.next()).await {
                Ok(Some((id, value))) => {
                    let mut domain = Signal {
                        source: id as u32,
                        timestamp: SystemTime::now(),
                        value,
                        last: None,
                    };

                    trace!("Source {} ⇨ {}", domain.source, domain.value);

                    if let Some(writer) = self.trace_writer.as_mut() {
                        let domain_systime = domain
                            .timestamp
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap();

                        match domain.value {
                            MetricValue::Temperature(scalar) => writer
                                .write_record(&[
                                    domain_systime.as_millis().to_string(),
                                    domain.source.to_string(),
                                    scalar.to_string(),
                                ])
                                .unwrap(),
                            MetricValue::Acceleration(vector) => writer
                                .write_record(&[
                                    domain_systime.as_millis().to_string(),
                                    domain.source.to_string(),
                                    vector.x.to_string(),
                                    vector.y.to_string(),
                                    vector.z.to_string(),
                                ])
                                .unwrap(),
                        }

                        // Best effort to reduce I/O.
                        if domain_systime.as_secs() % 5 == 0 {
                            writer.flush().unwrap();
                        }
                    }

                    domain.last = self
                        .cache
                        .insert(domain.source, domain.clone())
                        .map_or(None, |last_domain| Some(std::rc::Rc::new(last_domain)));

                    sink.distribute(domain);
                }
                Ok(None) => {}
                Err(_) => warn!("Timeout occured while reading from metric device"),
            }
        }
    }
}
