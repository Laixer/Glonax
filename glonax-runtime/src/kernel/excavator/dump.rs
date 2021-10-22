use glonax_core::{metric::MetricValue, motion::Motion};

use crate::runtime::{operand::*, Domain};

pub struct DumpProgram(Option<csv::Writer<std::fs::File>>);

impl DumpProgram {
    pub fn new() -> Self {
        Self(None)
    }
}

impl Program for DumpProgram {
    fn boot(&mut self, context: &mut Context) {
        let mut wtr = csv::Writer::from_path(context.session.path.join("metric.csv")).unwrap();

        wtr.write_record(&[
            "Device",
            "Acceleration X",
            "Acceleration Y",
            "Acceleration Z",
        ])
        .unwrap();

        self.0 = Some(wtr);
    }

    fn push(&mut self, domain: Domain) {
        trace!("Source {} ⇨ {}", domain.source, domain.value);

        match domain.value {
            MetricValue::Temperature(_) => (),
            MetricValue::Acceleration(vector) => {
                self.0
                    .as_mut()
                    .unwrap()
                    .write_record(&[
                        domain.source.to_string(),
                        vector.x.to_string(),
                        vector.y.to_string(),
                        vector.z.to_string(),
                    ])
                    .unwrap();
            }
        }

        self.0.as_mut().unwrap().flush().unwrap();
    }

    fn step(&mut self, context: &mut Context) -> Option<Motion> {
        trace!("Last step: {:?}", context.last_step.elapsed());

        None
    }

    fn can_terminate(&self, _context: &mut Context) -> bool {
        false
    }
}
