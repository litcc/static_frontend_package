use anyhow::{Error,Result};
use log::{Record, Level, Metadata, LevelFilter};


struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}


static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<()> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info)).map_err(|e|{
        Error::msg(format!("日志设置错误 {:?}",e))
    })
}