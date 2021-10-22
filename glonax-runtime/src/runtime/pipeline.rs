use std::time::{Duration, Instant};

use glonax_core::metric::MetricValue;

use crate::device::{DeviceDescriptor, MetricDevice};

#[derive(Debug, Clone)]
pub struct Domain {
    pub source: u32,
    pub timestamp: std::time::Instant,
    pub value: MetricValue,
    pub last: Option<std::rc::Rc<Domain>>,
}

pub trait Sink {
    fn distribute(&mut self, domain: Domain);
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

            wtr.write_record(&[
                "Source",
                "Acceleration X",
                "Acceleration Y",
                "Acceleration Z",
            ])
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
    cache: std::collections::HashMap<u32, Domain>,
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
                    let mut domain = Domain {
                        source: id as u32,
                        timestamp: Instant::now(),
                        value,
                        last: None,
                    };

                    trace!("Source {} ⇨ {}", domain.source, domain.value);

                    if let Some(writer) = self.trace_writer.as_mut() {
                        match domain.value {
                            MetricValue::Temperature(_) => (),
                            MetricValue::Acceleration(vector) => {
                                writer
                                    .write_record(&[
                                        domain.source.to_string(),
                                        vector.x.to_string(),
                                        vector.y.to_string(),
                                        vector.z.to_string(),
                                    ])
                                    .unwrap();
                            }
                        }

                        writer.flush().unwrap();
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
