use std::path::{Path, PathBuf};

use crate::core::{TraceWriter, Tracer};

pub struct NullTracer;

impl Tracer for NullTracer {
    type Instance = NullTracerInstance;

    fn from_path<P: AsRef<std::path::Path>>(_: P) -> Self {
        Self {}
    }

    fn instance(&self, _: &str) -> Self::Instance {
        Self::Instance {}
    }
}
pub struct NullTracerInstance;

impl TraceWriter for NullTracerInstance {
    fn write_record<T: serde::Serialize>(&mut self, _: T) {}
}

pub struct CsvTracer(PathBuf);

impl Tracer for CsvTracer {
    type Instance = CsvTracerInstance;

    fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self(path.as_ref().to_path_buf())
    }

    fn instance(&self, name: &str) -> Self::Instance {
        let writer = csv::WriterBuilder::new()
            .quote_style(csv::QuoteStyle::NonNumeric)
            .has_headers(true)
            .from_path(self.0.join(name.to_owned() + "_trace0.csv"))
            .unwrap();

        Self::Instance { writer }
    }
}

pub struct CsvTracerInstance {
    writer: csv::Writer<std::fs::File>,
}

impl TraceWriter for CsvTracerInstance {
    fn write_record<T: serde::Serialize>(&mut self, record: T) {
        self.writer.serialize(record).unwrap();
    }
}
