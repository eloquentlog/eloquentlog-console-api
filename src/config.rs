use std::env;

#[derive(Clone)]
pub struct Config {
    pub activation_ticket_issuer: String,
    pub activation_ticket_key_id: String,
    pub activation_ticket_secret: String,
    pub authorization_ticket_issuer: String,
    pub authorization_ticket_key_id: String,
    pub authorization_ticket_secret: String,
    pub database_url: String,
    pub env_name: &'static str,
    pub queue_url: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            activation_ticket_issuer: env::var("ACTIVATION_TICKET_ISSUER")
                .expect("ACTIVATION_TICKET_ISSUER is not set"),
            activation_ticket_key_id: env::var("ACTIVATION_TICKET_KEY_ID")
                .expect("ACTIVATION_TICKET_KEY_ID is not set"),
            activation_ticket_secret: env::var("ACTIVATION_TICKET_SECRET")
                .expect("ACTIVATION_TICKET_SECRET is not set"),

            authorization_ticket_issuer: env::var(
                "AUTHORIZATION_TICKET_ISSUER",
            )
            .expect("AUTHORIZATION_TICKET_ISSUER is not set"),
            authorization_ticket_key_id: env::var(
                "AUTHORIZATION_TICKET_KEY_ID",
            )
            .expect("AUTHORIZATION_TICKET_KEY_ID is not set"),
            authorization_ticket_secret: env::var(
                "AUTHORIZATION_TICKET_SECRET",
            )
            .expect("AUTHORIZATION_TICKET_SECRET is not set"),

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
            activation_ticket_issuer: env::var("TEST_ACTIVATION_TICKET_ISSUER")
                .expect("TEST_ACTIVATION_TICKET_ISSUER is not set"),
            activation_ticket_key_id: env::var("TEST_ACTIVATION_TICKET_KEY_ID")
                .expect("TEST_ACTIVATION_TICKET_KEY_ID is not set"),
            activation_ticket_secret: env::var("TEST_ACTIVATION_TICKET_SECRET")
                .expect("TEST_ACTIVATION_TICKET_SECRET is not set"),

            authorization_ticket_issuer: env::var(
                "TEST_AUTHORIZATION_TICKET_ISSUER",
            )
            .expect("TEST_AUTHORIZATION_TICKET_ISSUER is not set"),
            authorization_ticket_key_id: env::var(
                "TEST_AUTHORIZATION_TICKET_KEY_ID",
            )
            .expect("TEST_AUTHORIZATION_TICKET_KEY_ID is not set"),
            authorization_ticket_secret: env::var(
                "TEST_AUTHORIZATION_TICKET_SECRET",
            )
            .expect("TEST_AUTHORIZATION_TICKET_SECRET is not set"),

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
mod test {
    use super::*;

    use std::collections::HashMap;
    use std::panic::{self, AssertUnwindSafe};

    use parking_lot::Mutex;

    use hashmap;

    // TODO: set HashMap as an arg
    fn with<T>(keys: &'static str, test: T)
    where T: FnOnce() -> () + panic::UnwindSafe {
        lazy_static! {
            static ref ENV_LOCK: Mutex<()> = Mutex::new(());
            static ref TESTS: HashMap<&'static str, &'static str> = hashmap! {
                "ACTIVATION_TICKET_ISSUER" => "com.eloquentlog",
                "ACTIVATION_TICKET_KEY_ID" => "key_id-activation",
                "ACTIVATION_TICKET_SECRET" => "secret-activation",
                "AUTHORIZATION_TICKET_ISSUER" => "com.eloquentlog",
                "AUTHORIZATION_TICKET_KEY_ID" => "key_id-authorization",
                "AUTHORIZATION_TICKET_SECRET" => "secret-authorization",
                "DATABASE_URL" =>
                    "postgresql://u$er:pa$$w0rd@localhost:5432/dbname",
                "QUEUE_URL" => "redis://u$er:pa$$w0rd@localhost:6379/queue",

                "TEST_ACTIVATION_TICKET_ISSUER" => "com.eloquentlog",
                "TEST_ACTIVATION_TICKET_KEY_ID" => "test-key_id-activation",
                "TEST_ACTIVATION_TICKET_SECRET" => "test-secret-activation",
                "TEST_AUTHORIZATION_TICKET_ISSUER" => "com.eloquentlog",
                "TEST_AUTHORIZATION_TICKET_KEY_ID" => "test-key_id-authorization",
                "TEST_AUTHORIZATION_TICKET_SECRET" => "test-secret-authorization",
                "TEST_DATABASE_URL" =>
                    "postgresql://u$er:pa$$w0rd@localhost:5432/dbname",
                "TEST_QUEUE_URL" => "redis://u$er:pa$$w0rd@localhost:6379/queue"
            };
        }

        let _lock = ENV_LOCK.lock();

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut origins: HashMap<&str, Result<String, env::VarError>> =
                HashMap::new();

            let inputs: Vec<&str> = keys.split('\n').collect();
            for (key, var) in TESTS.iter() {
                origins.insert(key, env::var(key));

                if !inputs.contains(&key) {
                    env::remove_var(key);
                } else {
                    env::set_var(key, var);
                }
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

    #[test]
    fn test_from_unknown_without_env_vars() {
        let c = Config::from("unknown");
        assert!(c.is_err());
    }

    rusty_fork_test! {
        #[test]
        fn test_from_production_without_valid_env_vars() {
            with(r#"
TEST_ACTIVATION_TICKET_ISSUER
TEST_ACTIVATION_TICKET_KEY_ID
TEST_ACTIVATION_TICKET_SECRET
TEST_AUTHORIZATION_TICKET_ISSUER
TEST_AUTHORIZATION_TICKET_KEY_ID
TEST_AUTHORIZATION_TICKET_SECRET
TEST_DATABASE_URL
TEST_QUEUE_URL
"#, || {
                let result = panic::catch_unwind(|| {
                    let c = Config::from("production");
                    assert!(c.is_ok());
                });
                assert!(result.is_err());
            })
        }
    }

    rusty_fork_test! {
        #[test]
        fn test_from_testing_without_valid_env_vars() {
            with(r#"
ACTIVATION_TICKET_ISSUER
ACTIVATION_TICKET_KEY_ID
ACTIVATION_TICKET_SECRET
AUTHORIZATION_TICKET_ISSUER
AUTHORIZATION_TICKET_KEY_ID
AUTHORIZATION_TICKET_SECRET
DATABASE_URL
QUEUE_URL
"#, || {
                let result = panic::catch_unwind(|| {
                    let c = Config::from("testing");
                    assert!(c.is_ok());
                });
                assert!(result.is_err());
            })
        }
    }

    rusty_fork_test! {
        #[test]
        fn test_from_development_without_valid_env_vars() {
            with(r#"
TEST_ACTIVATION_TICKET_ISSUER
TEST_ACTIVATION_TICKET_KEY_ID
TEST_ACTIVATION_TICKET_SECRET
TEST_AUTHORIZATION_TICKET_ISSUER
TEST_AUTHORIZATION_TICKET_KEY_ID
TEST_AUTHORIZATION_TICKET_SECRET
TEST_DATABASE_URL
TEST_QUEUE_URL
"#, || {
                let result = panic::catch_unwind(|| {
                    let c = Config::from("development");
                    assert!(c.is_ok());
                });
                assert!(result.is_err());
            })
        }
    }

    rusty_fork_test! {
        #[test]
        fn test_from_production() {
            with(r#"
ACTIVATION_TICKET_ISSUER
ACTIVATION_TICKET_KEY_ID
ACTIVATION_TICKET_SECRET
AUTHORIZATION_TICKET_ISSUER
AUTHORIZATION_TICKET_KEY_ID
AUTHORIZATION_TICKET_SECRET
DATABASE_URL
QUEUE_URL
"#, || {
                let c = Config::from("production");
                assert_eq!(c.unwrap().env_name, "production");
            });
        }
    }

    rusty_fork_test! {
        #[test]
        fn test_from_testing() {
            with(r#"
TEST_ACTIVATION_TICKET_ISSUER
TEST_ACTIVATION_TICKET_KEY_ID
TEST_ACTIVATION_TICKET_SECRET
TEST_AUTHORIZATION_TICKET_ISSUER
TEST_AUTHORIZATION_TICKET_KEY_ID
TEST_AUTHORIZATION_TICKET_SECRET
TEST_DATABASE_URL
TEST_QUEUE_URL
"#, || {
                let c = Config::from("testing");
                assert_eq!(c.unwrap().env_name, "testing");
            });
        }
    }

    rusty_fork_test! {
        #[test]
        fn test_from_development() {
            with(r#"
ACTIVATION_TICKET_ISSUER
ACTIVATION_TICKET_KEY_ID
ACTIVATION_TICKET_SECRET
AUTHORIZATION_TICKET_ISSUER
AUTHORIZATION_TICKET_KEY_ID
AUTHORIZATION_TICKET_SECRET
DATABASE_URL
QUEUE_URL
"#, || {
                let c = Config::from("development");
                assert_eq!(c.unwrap().env_name, "development");
            });
        }
    }
}
