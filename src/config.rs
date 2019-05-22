use std::env;

use dotenv::dotenv;

pub struct Config {
    pub database_url: String,
    pub env_name: &'static str,
    pub jwt_issuer: String,
    pub jwt_secret: String,
    pub queue_url: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL is not set"),
            env_name: &"undefined",
            jwt_issuer: env::var("JWT_ISSUER").expect("JWT_ISSUER is not set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET is not set"),
            queue_url: env::var("QUEUE_URL").expect("QUEUE_URL is not set"),
        }
    }
}

impl Config {
    pub fn from(config_name: &str) -> Result<Config, String> {
        dotenv().ok();

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
            jwt_issuer: env::var("TEST_JWT_ISSUER")
                .expect("TEST_JWT_ISSUER is not set"),
            jwt_secret: env::var("TEST_JWT_SECRET")
                .expect("TEST_JWT_SECRET is not set"),
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
    use super::*;

    use std::collections::HashMap;
    use std::panic::{self, AssertUnwindSafe};

    use parking_lot::Mutex;

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
            static ref ENV_LOCK: Mutex<()> = Mutex::new(());
            static ref DEFAULTS: HashMap<&'static str, &'static str> = map! {
                "DATABASE_URL" =>
                    "postgresql://u$er:pa$$w0rd@localhost:5432/dbname",
                "JWT_ISSUER" => "com.eloquentlog",
                "JWT_SECRET" => "secret",
                "QUEUE_URL" => "redis://u$er:pa$$w0rd@localhost:6379/queue",

                "TEST_DATABASE_URL" =>
                    "postgresql://u$er:pa$$w0rd@localhost:5432/dbname",
                "TEST_JWT_ISSUER" => "com.eloquentlog",
                "TEST_JWT_SECRET" => "test-secret",
                "TEST_QUEUE_URL" => "redis://u$er:pa$$w0rd@localhost:6379/queue"
            };
        }

        let _lock = ENV_LOCK.lock();

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut origins: HashMap<&str, Result<String, env::VarError>> =
                HashMap::new();

            let inputs: Vec<&str> = keys.split(',').collect();
            for key in inputs {
                origins.insert(key, env::var(key));

                let var = DEFAULTS.get(key).unwrap();
                env::set_var(key, var);
            }

            test();

            for (key, origin) in origins {
                match origin {
                    Ok(v) => env::set_var(key, v),
                    Err(_) => env::remove_var(key),
                }
            }
        }));

        assert!(result.is_ok());
    }

    rusty_fork_test! {
        #[test]
        fn test_from() {
            // without any env vars
            let c = Config::from("unknown");
            assert!(c.is_err());

            // with TEST_ prefix
            with_env_vars(
                "TEST_DATABASE_URL,TEST_JWT_ISSUER,TEST_JWT_SECRET,TEST_QUEUE_URL",
                || {
                let c = Config::from("unknown");
                assert!(c.is_err());

                let c = Config::from("testing");
                assert_eq!(c.unwrap().env_name, "testing");
            });

            // production or development
            with_env_vars("DATABASE_URL,JWT_ISSUER,JWT_SECRET,QUEUE_URL", || {
                let c = Config::from("unknown");
                assert!(c.is_err());

                let c = Config::from("development");
                assert_eq!(c.unwrap().env_name, "development");

                let c = Config::from("production");
                assert_eq!(c.unwrap().env_name, "production");
            });
        }
    }
}
