use log::{Level, Log, Metadata, Record};

pub struct SystemdLogger;

impl Log for SystemdLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let level = match record.level() {
            Level::Error => "<3>",
            Level::Warn => "<4>",
            Level::Info => "<6>",
            Level::Debug => "<7>",
            Level::Trace => "<7>",
        };

        if record.level() == Level::Error {
            eprintln!("{}{}", level, record.args());
        } else {
            println!("{}{}", level, record.args());
        }
    }

    fn flush(&self) {}
}
