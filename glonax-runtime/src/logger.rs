use log::{Level, Log, Metadata, Record};

pub struct SystemdLogger;

/// Implementation of the `Log` trait for the `SystemdLogger` struct.
impl Log for SystemdLogger {
    /// Determines if logging is enabled for the given metadata.
    ///
    /// This function always returns `true`.
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    /// Logs the given record.
    ///
    /// The log level is converted to a corresponding systemd log level and
    /// printed along with the log message.
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

    /// Flushes any buffered log records.
    ///
    /// This function does nothing.
    fn flush(&self) {}
}
