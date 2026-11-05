use std::{
  fmt::Display,
  fs::File,
  io::Write,
  sync::{Mutex, OnceLock},
};

use chrono::Local;
use colored::Colorize;

pub enum LogKind {
  Info,
  Success,
  Warn,
  Error,
}

/// Release builds have no console on Windows, so set `LUMINA_LOG_FILE` to a path to keep a log.
static LOG_FILE: OnceLock<Option<Mutex<File>>> = OnceLock::new();

fn log_file() -> Option<&'static Mutex<File>> {
  LOG_FILE
    .get_or_init(|| {
      let path = std::env::var("LUMINA_LOG_FILE").ok()?;
      File::create(path).ok().map(Mutex::new)
    })
    .as_ref()
}

pub fn log(s: impl AsRef<str> + Display, kind: Option<LogKind>) {
  let (status, plain) = match kind {
    Some(LogKind::Info) => ("INFO".blue(), "INFO"),
    Some(LogKind::Success) => ("DONE".green(), "DONE"),
    Some(LogKind::Warn) => ("WARN".yellow(), "WARN"),
    Some(LogKind::Error) => ("FAIL".red(), "FAIL"),
    None => ("INFO".blue(), "INFO"),
  };

  let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

  let _ = writeln!(
    std::io::stdout().lock(),
    "[{}] [{}] {}",
    timestamp,
    status,
    s
  );

  if let Some(file) = log_file()
    && let Ok(mut file) = file.lock()
  {
    let _ = writeln!(file, "[{}] [{}] {}", timestamp, plain, s);
  }
}

#[macro_export]
macro_rules! log {
  ($($arg:tt)*) => {
    $crate::logger::log(format!($($arg)*), Some($crate::logger::LogKind::Info))
  };
}

#[macro_export]
macro_rules! success {
  ($($arg:tt)*) => {
    $crate::logger::log(format!($($arg)*), Some($crate::logger::LogKind::Success))
  };
}

#[macro_export]
macro_rules! warn {
  ($($arg:tt)*) => {
    $crate::logger::log(format!($($arg)*), Some($crate::logger::LogKind::Warn))
  };
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => {
    $crate::logger::log(format!($($arg)*), Some($crate::logger::LogKind::Error))
  };
}
