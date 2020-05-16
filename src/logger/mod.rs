use crate::{println, serial_println};
use log::{Metadata, Record};

struct SimpleLogger;

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() {
    log::set_logger(&LOGGER).expect("failed to set logger");
    log::set_max_level(log::LevelFilter::Trace);
}

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        serial_println!("[{} {}:{}] {}", record.level(), record.file().unwrap(), record.line().unwrap(), record.args());
        println!("[{} {}:{}] {}", record.level(), record.file().unwrap(), record.line().unwrap(), record.args());
    }

    fn flush(&self) {}
}
