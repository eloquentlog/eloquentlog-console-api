use rocket_slog::SlogFairing;

use sloggers::{
    Build,
    terminal::{TerminalLoggerBuilder, Destination},
    types::Severity,
};

use config::Config;

pub fn get_logger(config: &Config) -> SlogFairing {
    let mut builder = TerminalLoggerBuilder::new();

    let level = match config.env_name {
        "development" => Severity::Debug,
        "testing" => Severity::Error,
        _ => Severity::Critical,
    };

    builder.level(level);
    builder.destination(Destination::Stdout);
    let logger = builder.build().unwrap();

    SlogFairing::new(logger)
}
