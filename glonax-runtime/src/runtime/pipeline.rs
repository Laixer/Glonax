use std::time::{Duration, Instant};

use glonax_core::metric::MetricValue;

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

pub(super) struct Pipeline<'a> {
    source_list:
        &'a mut Vec<crate::device::DeviceDescriptor<dyn crate::device::MetricDevice + Send>>,
    cache: std::collections::HashMap<u32, Domain>,
}

unsafe impl Send for Pipeline<'_> {}

impl<'a> Pipeline<'a> {
    pub(super) fn new(
        source_list: &'a mut Vec<
            crate::device::DeviceDescriptor<dyn crate::device::MetricDevice + Send>,
        >,
    ) -> Self {
        Self {
            source_list,
            cache: std::collections::HashMap::new(),
        }
    }

    // FUTURE: lock all devices at the same time.
    pub(super) async fn push_all<T: Sink + ?Sized>(&mut self, sink: &mut T, timeout: Duration) {
        for metric_device in self.source_list.iter_mut() {
            // Take up to 5ms until this read is cancelled and we move to the next device.
            match tokio::time::timeout(timeout, metric_device.lock().await.next()).await {
                Ok(Some((id, value))) => {
                    let mut domain = Domain {
                        source: id as u32,
                        timestamp: Instant::now(),
                        value,
                        last: None,
                    };

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
