use sloggers::{
    Build,
    terminal::{TerminalLoggerBuilder, Destination},
    types::Severity,
};

use crate::config::Config;

pub type Logger = slog::Logger;

pub fn get_logger(config: &Config) -> Logger {
    let mut builder = TerminalLoggerBuilder::new();

    let level = match config.env_name {
        "development" => Severity::Debug,
        "production" => Severity::Error,
        "testing" => Severity::Warning,
        _ => Severity::Trace,
    };

    builder.level(level);
    builder.destination(Destination::Stdout);
    builder.build().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    use slog::Drain;

    use crate::model::test::run;

    #[test]
    fn test_get_logger_development() {
        run(|_, config, _| {
            let mut c = config.clone();
            c.env_name = "development";
            let logger = get_logger(&c);

            assert!(logger.is_critical_enabled());
            assert!(logger.is_error_enabled());
            assert!(logger.is_warning_enabled());
            assert!(logger.is_info_enabled());
            assert!(logger.is_debug_enabled());
            assert!(!logger.is_trace_enabled());
        })
    }

    #[test]
    fn test_get_logger_production() {
        run(|_, config, _| {
            let mut c = config.clone();
            c.env_name = "production";
            let logger = get_logger(&c);

            assert!(logger.is_critical_enabled());
            assert!(logger.is_error_enabled());
            assert!(!logger.is_warning_enabled());
            assert!(!logger.is_info_enabled());
            assert!(!logger.is_debug_enabled());
            assert!(!logger.is_trace_enabled());
        })
    }

    #[test]
    fn test_get_logger_testing() {
        run(|_, config, _| {
            let logger = get_logger(config);

            assert!(logger.is_critical_enabled());
            assert!(logger.is_error_enabled());
            assert!(logger.is_warning_enabled());
            assert!(!logger.is_info_enabled());
            assert!(!logger.is_debug_enabled());
            assert!(!logger.is_trace_enabled());
        })
    }
}
