use std::env;

pub struct Config {
    pub database_url: String,
    pub env_name: &'static str,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL is not set"),
            env_name: &"undefined",
        }
    }
}

impl Config {
    pub fn from(config_name: &str) -> Result<Config, String> {
        match config_name {
            "production" => Ok(Config::production_config()),
            "testing" => Ok(Config::testing_config()),
            "development" => Ok(Config::development_config()),
            _ => Err(format!("Invalid config_name: '{}'", &config_name)),
        }
    }

    fn production_config() -> Config {
        Config {
            env_name: &"production",
            ..Default::default()
        }
    }

    fn testing_config() -> Config {
        Config {
            database_url: env::var("TEST_DATABASE_URL")
                .expect("TEST_DATABASE_URL is not set"),
            env_name: &"testing",
        }
    }

    fn development_config() -> Config {
        Config {
            env_name: &"development",
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod config_test {
    use std::panic;

    use super::*;

    fn with_database_url<T>(key: &'static str, test: T)
    where T: FnOnce() -> () + panic::UnwindSafe {
        let orig = env::var(key);
        env::set_var(key, "postgresql://u$er:pa$$word@localhost:5432/dbname");
        let result = panic::catch_unwind(test);

        match orig {
            Ok(v) => env::set_var(key, v),
            Err(_) => env::remove_var(key),
        }

        assert!(result.is_ok());
    }

    #[test]
    fn test_from() {
        // without DATABASE_URL
        let c = Config::from("unknown");
        assert!(c.is_err());

        with_database_url("TEST_DATABASE_URL", || {
            let c = Config::from("testing");
            assert_eq!(c.unwrap().env_name, "testing");
        });

        with_database_url("DATABASE_URL", || {
            let c = Config::from("unknown");
            assert!(c.is_err());

            let c = Config::from("development");
            assert_eq!(c.unwrap().env_name, "development");

            let c = Config::from("production");
            assert_eq!(c.unwrap().env_name, "production");
        });
    }
}
