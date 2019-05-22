use sloggers::{
    Build,
    terminal::{TerminalLoggerBuilder, Destination},
    types::Severity,
};

use config::Config;

pub type Logger = slog::Logger;

pub fn get_logger(config: &Config) -> Logger {
    let mut builder = TerminalLoggerBuilder::new();

    let level = match config.env_name {
        "development" => Severity::Debug,
        "testing" => Severity::Error,
        _ => Severity::Critical,
    };

    builder.level(level);
    builder.destination(Destination::Stdout);
    builder.build().unwrap()
}
