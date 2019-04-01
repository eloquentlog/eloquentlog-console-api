use std::env;

pub struct Config {
    pub database_url: String,
    pub env_name: &'static str,
    pub queue_url: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL is not set"),
            env_name: &"undefined",
            queue_url: env::var("QUEUE_URL").expect("QUEUE_URL is not set"),
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
            queue_url: env::var("TEST_QUEUE_URL")
                .expect("TEST_QUEUE_URL is not set"),
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
    use std::collections::HashMap;
    use std::panic;

    use super::*;

    macro_rules! map(
        { $($key:expr => $value:expr),+ } => {
            {
                let mut m = ::std::collections::HashMap::new();
                $(
                    m.insert($key, $value);
                )+
                m
            }
        };
    );

    // TODO: set HashMap as an arg
    fn with_env_vars<T>(keys: &'static str, test: T)
    where T: FnOnce() -> () + panic::UnwindSafe {
        lazy_static! {
            static ref DEFAULTS: HashMap<&'static str, &'static str> = map! {
                "DATABASE_URL" =>
                    "postgresql://u$er:pa$$w0rd@localhost:5432/dbname",
                "QUEUE_URL" => "redis://u$er:pa$$w0rd@localhost:6379/queue",

                "TEST_DATABASE_URL" =>
                    "postgresql://u$er:pa$$w0rd@localhost:5432/dbname",
                "TEST_QUEUE_URL" => "redis://u$er:pa$$w0rd@localhost:6379/queue"
            };
        }

        let mut origins: HashMap<&str, Result<String, env::VarError>> =
            HashMap::new();

        let inputs: Vec<&str> = keys.split(',').collect();
        for key in inputs {
            origins.insert(key, env::var(key));

            let var = DEFAULTS.get(key).unwrap();
            env::set_var(key, var);
        }

        let result = panic::catch_unwind(test);

        for (key, origin) in origins {
            match origin {
                Ok(v) => env::set_var(key, v),
                Err(_) => env::remove_var(key),
            }
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_from() {
        // without any env vars
        let c = Config::from("unknown");
        assert!(c.is_err());

        // with TEST_ prefix
        with_env_vars("TEST_DATABASE_URL,TEST_QUEUE_URL", || {
            let c = Config::from("unknown");
            assert!(c.is_err());

            let c = Config::from("testing");
            assert_eq!(c.unwrap().env_name, "testing");
        });

        // production or development
        with_env_vars("DATABASE_URL,QUEUE_URL", || {
            let c = Config::from("unknown");
            assert!(c.is_err());

            let c = Config::from("development");
            assert_eq!(c.unwrap().env_name, "development");

            let c = Config::from("production");
            assert_eq!(c.unwrap().env_name, "production");
        });
    }
}
