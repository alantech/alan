use chrono::{DateTime, Local, Utc};
use log::{set_boxed_logger, Level, LevelFilter, Metadata, Record, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= Level::Info
  }

  fn log(&self, record: &Record) {
    let local_time = Local::now();
    let utc_time = DateTime::<Utc>::from_utc(local_time.naive_utc(), Utc);
    if self.enabled(record.metadata()) {
      if record.level() == Level::Error {
        eprintln!("{} | {} | {}", utc_time, record.level(), record.args());
      } else {
        println!("{} | {} | {}", utc_time, record.level(), record.args());
      }
    }
  }

  fn flush(&self) {}
}

fn config_logger_local(_: ()) {
  log::set_max_level(LevelFilter::Trace);
}

fn config_logger(_: ()) {
  log::set_max_level(LevelFilter::Info);
}

pub fn init() -> Result<(), SetLoggerError> {
  let env = std::env::var("ALAN_TECH_ENV").unwrap_or("production".to_string());
  match env.as_str() {
    "local" => set_boxed_logger(Box::new(SimpleLogger)).map(config_logger_local),
    _ => set_boxed_logger(Box::new(SimpleLogger)).map(config_logger), // TODO: update with new logger struct once decide
  }
}
