use std::env;

#[derive(Clone)]
pub struct Config {
    pub activation_token_issuer: String,
    pub activation_token_key_id: String,
    pub activation_token_secret: String,
    pub authorization_token_issuer: String,
    pub authorization_token_key_id: String,
    pub authorization_token_secret: String,
    pub application_url: String,
    pub database_url: String,
    pub database_max_pool_size: u32,
    pub env_name: &'static str,
    pub mailer_domain: String,
    pub mailer_from_email: String,
    pub mailer_from_alias: String,
    pub mailer_smtp_host: String,
    pub mailer_smtp_port: u16,
    pub mailer_smtp_username: String,
    pub mailer_smtp_password: String,
    pub message_queue_url: String,
    pub message_queue_max_pool_size: u32,
    pub session_store_url: String,
    pub session_store_max_pool_size: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            activation_token_issuer: env::var("ACTIVATION_TOKEN_ISSUER")
                .expect("ACTIVATION_TOKEN_ISSUER is not set"),
            activation_token_key_id: env::var("ACTIVATION_TOKEN_KEY_ID")
                .expect("ACTIVATION_TOKEN_KEY_ID is not set"),
            activation_token_secret: env::var("ACTIVATION_TOKEN_SECRET")
                .expect("ACTIVATION_TOKEN_SECRET is not set"),

            authorization_token_issuer: env::var("AUTHORIZATION_TOKEN_ISSUER")
                .expect("AUTHORIZATION_TOKEN_ISSUER is not set"),
            authorization_token_key_id: env::var("AUTHORIZATION_TOKEN_KEY_ID")
                .expect("AUTHORIZATION_TOKEN_KEY_ID is not set"),
            authorization_token_secret: env::var("AUTHORIZATION_TOKEN_SECRET")
                .expect("AUTHORIZATION_TOKEN_SECRET is not set"),

            application_url: env::var("APPLICATION_URL")
                .expect("APPLICATION_URL is not set"),

            database_max_pool_size: 0,
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL is not set"),

            env_name: &"undefined",

            mailer_domain: env::var("MAILER_DOMAIN")
                .expect("MAILER_DOMAIN is not set"),
            mailer_from_email: env::var("MAILER_FROM_EMAIL")
                .expect("MAILER_FROM_EMAIL is not set"),
            mailer_from_alias: env::var("MAILER_FROM_ALIAS")
                .expect("MAILER_FROM_ALIAS is not set"),
            mailer_smtp_host: env::var("MAILER_SMTP_HOST")
                .expect("MAILER_SMTP_HOST is not set"),
            mailer_smtp_port: 587,
            mailer_smtp_username: env::var("MAILER_SMTP_USERNAME")
                .expect("MAILER_SMTP_USERNAME is not set"),
            mailer_smtp_password: env::var("MAILER_SMTP_PASSWORD")
                .expect("MAILER_SMTP_PASSWORD is not set"),

            message_queue_max_pool_size: 0,
            message_queue_url: env::var("MESSAGE_QUEUE_URL")
                .expect("MESSAGE_QUEUE_URL is not set"),

            session_store_max_pool_size: 0,
            session_store_url: env::var("SESSION_STORE_URL")
                .expect("SESSION_STORE_URL is not set"),
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
        let database_max_pool_size: u32 =
            match env::var("DATABASE_MAX_POOL_SIZE") {
                Ok(v) => v.parse::<u32>().unwrap(),
                Err(_) => 12,
            };

        let mailer_smtp_port: u16 = match env::var("MAILER_SMTP_PORT") {
            Ok(v) => v.parse::<u16>().unwrap(),
            Err(_) => 587,
        };

        let message_queue_max_pool_size: u32 =
            match env::var("MESSAGE_QUEUE_MAX_POOL_SIZE") {
                Ok(v) => v.parse::<u32>().unwrap(),
                Err(_) => 8,
            };

        let session_store_max_pool_size: u32 =
            match env::var("SESSION_STORE_MAX_POOL_SIZE") {
                Ok(v) => v.parse::<u32>().unwrap(),
                Err(_) => 8,
            };

        Config {
            env_name: &"production",
            database_max_pool_size,
            mailer_smtp_port,
            message_queue_max_pool_size,
            session_store_max_pool_size,

            ..Default::default()
        }
    }

    // NOTE:
    // xxx_max_pool_size both must be >= 2 for integration tests.
    // Because the pool will be shared between the server and a client for the
    // instance.
    fn testing_config() -> Config {
        let database_max_pool_size: u32 =
            match env::var("TEST_DATABASE_MAX_POOL_SIZE") {
                Ok(v) => v.parse::<u32>().unwrap(),
                Err(_) => 2,
            };

        let mailer_smtp_port: u16 = match env::var("TEST_MAILER_SMTP_PORT") {
            Ok(v) => v.parse::<u16>().unwrap(),
            Err(_) => 587,
        };

        let message_queue_max_pool_size: u32 =
            match env::var("TEST_MESSAGE_QUEUE_MAX_POOL_SIZE") {
                Ok(v) => v.parse::<u32>().unwrap(),
                Err(_) => 2,
            };

        let session_store_max_pool_size: u32 =
            match env::var("TEST_SESSION_STORE_MAX_POOL_SIZE") {
                Ok(v) => v.parse::<u32>().unwrap(),
                Err(_) => 2,
            };

        Config {
            activation_token_issuer: env::var("TEST_ACTIVATION_TOKEN_ISSUER")
                .expect("TEST_ACTIVATION_TOKEN_ISSUER is not set"),
            activation_token_key_id: env::var("TEST_ACTIVATION_TOKEN_KEY_ID")
                .expect("TEST_ACTIVATION_TOKEN_KEY_ID is not set"),
            activation_token_secret: env::var("TEST_ACTIVATION_TOKEN_SECRET")
                .expect("TEST_ACTIVATION_TOKEN_SECRET is not set"),

            authorization_token_issuer: env::var(
                "TEST_AUTHORIZATION_TOKEN_ISSUER",
            )
            .expect("TEST_AUTHORIZATION_TOKEN_ISSUER is not set"),
            authorization_token_key_id: env::var(
                "TEST_AUTHORIZATION_TOKEN_KEY_ID",
            )
            .expect("TEST_AUTHORIZATION_TOKEN_KEY_ID is not set"),
            authorization_token_secret: env::var(
                "TEST_AUTHORIZATION_TOKEN_SECRET",
            )
            .expect("TEST_AUTHORIZATION_TOKEN_SECRET is not set"),

            application_url: env::var("TEST_APPLICATION_URL")
                .expect("TEST_APPLICATION_URL is not set"),

            database_max_pool_size,
            database_url: env::var("TEST_DATABASE_URL")
                .expect("TEST_DATABASE_URL is not set"),

            env_name: &"testing",

            mailer_domain: env::var("TEST_MAILER_DOMAIN")
                .expect("TEST_MAILER_DOMAIN is not set"),
            mailer_from_email: env::var("TEST_MAILER_FROM_EMAIL")
                .expect("TEST_MAILER_FROM_EMAIL is not set"),
            mailer_from_alias: env::var("TEST_MAILER_FROM_ALIAS")
                .expect("TEST_MAILER_FROM_ALIAS is not set"),
            mailer_smtp_host: env::var("TEST_MAILER_SMTP_HOST")
                .expect("TEST_MAILER_SMTP_HOST is not set"),
            mailer_smtp_port,
            mailer_smtp_username: env::var("TEST_MAILER_SMTP_USERNAME")
                .expect("TEST_MAILER_SMTP_USERNAME is not set"),
            mailer_smtp_password: env::var("TEST_MAILER_SMTP_PASSWORD")
                .expect("TEST_MAILER_SMTP_PASSWORD is not set"),

            message_queue_max_pool_size,
            message_queue_url: env::var("TEST_MESSAGE_QUEUE_URL")
                .expect("TEST_MESSAGE_QUEUE_URL is not set"),

            session_store_max_pool_size,
            session_store_url: env::var("TEST_SESSION_STORE_URL")
                .expect("TEST_SESSION_STORE_URL is not set"),
        }
    }

    fn development_config() -> Config {
        let database_max_pool_size: u32 =
            match env::var("DATABASE_MAX_POOL_SIZE") {
                Ok(v) => v.parse::<u32>().unwrap(),
                Err(_) => 4,
            };

        let mailer_smtp_port: u16 = match env::var("MAILER_SMTP_PORT") {
            Ok(v) => v.parse::<u16>().unwrap(),
            Err(_) => 587,
        };

        let message_queue_max_pool_size: u32 =
            match env::var("MESSAGE_QUEUE_MAX_POOL_SIZE") {
                Ok(v) => v.parse::<u32>().unwrap(),
                Err(_) => 4,
            };

        let session_store_max_pool_size: u32 =
            match env::var("SESSION_STORE_MAX_POOL_SIZE") {
                Ok(v) => v.parse::<u32>().unwrap(),
                Err(_) => 4,
            };

        Config {
            env_name: &"development",
            database_max_pool_size,
            mailer_smtp_port,
            message_queue_max_pool_size,
            session_store_max_pool_size,

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
                "ACTIVATION_TOKEN_ISSUER" => "com.eloquentlog",
                "ACTIVATION_TOKEN_KEY_ID" => "key_id-activation",
                "ACTIVATION_TOKEN_SECRET" => "secret-activation",
                "AUTHORIZATION_TOKEN_ISSUER" => "com.eloquentlog",
                "AUTHORIZATION_TOKEN_KEY_ID" => "key_id-authorization",
                "AUTHORIZATION_TOKEN_SECRET" => "secret-authorization",
                "APPLICATION_URL" => "http://127.0.0.1:3000",
                "DATABASE_URL" =>
                    "postgresql://u$er:pa$$w0rd@localhost:5432/dbname",
                "MAILER_DOMAIN" => "eloquentlog.com",
                "MAILER_FROM_EMAIL" => "no-reply@eloquentlog.com",
                "MAILER_FROM_ALIAS" => "Eloquentlog - Development",
                "MAILER_SMTP_HOST" => "server.tld",
                "MAILER_SMTP_USERNAME" => "username",
                "MAILER_SMTP_PASSWORD" => "password",
                "MESSAGE_QUEUE_URL" =>
                    "redis://u$er:pa$$w0rd@localhost:6379/message",
                "SESSION_STORE_URL" =>
                    "redis://u$er:pa$$w0rd@localhost:6379/session",

                "TEST_ACTIVATION_TOKEN_ISSUER" => "com.eloquentlog",
                "TEST_ACTIVATION_TOKEN_KEY_ID" => "test-key_id-activation",
                "TEST_ACTIVATION_TOKEN_SECRET" => "test-secret-activation",
                "TEST_AUTHORIZATION_TOKEN_ISSUER" => "com.eloquentlog",
                "TEST_AUTHORIZATION_TOKEN_KEY_ID" => "test-key_id-authorization",
                "TEST_AUTHORIZATION_TOKEN_SECRET" => "test-secret-authorization",
                "TEST_APPLICATION_URL" => "http://127.0.0.1:3000",
                "TEST_DATABASE_URL" =>
                    "postgresql://u$er:pa$$w0rd@localhost:5432/dbname",
                "TEST_MAILER_DOMAIN" => "eloquentlog.com",
                "TEST_MAILER_FROM_EMAIL" => "no-reply@eloquentlog.com",
                "TEST_MAILER_FROM_ALIAS" => "Eloquentlog - Testing",
                "TEST_MAILER_SMTP_HOST" => "server.tld",
                "TEST_MAILER_SMTP_USERNAME" => "username",
                "TEST_MAILER_SMTP_PASSWORD" => "password",
                "TEST_MESSAGE_QUEUE_URL" =>
                    "redis://u$er:pa$$w0rd@localhost:6379/message",
                "TEST_SESSION_STORE_URL" =>
                    "redis://u$er:pa$$w0rd@localhost:6379/session"
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
TEST_ACTIVATION_TOKEN_ISSUER
TEST_ACTIVATION_TOKEN_KEY_ID
TEST_ACTIVATION_TOKEN_SECRET
TEST_AUTHORIZATION_TOKEN_ISSUER
TEST_AUTHORIZATION_TOKEN_KEY_ID
TEST_AUTHORIZATION_TOKEN_SECRET
TEST_APPLICATION_URL
TEST_DATABASE_URL
TEST_MAILER_DOMAIN
TEST_MAILER_FROM_EMAIL
TEST_MAILER_FROM_ALIAS
TEST_MAILER_SMTP_HOST
TEST_MAILER_SMTP_PASSWORD
TEST_MAILER_SMTP_USERNAME
TEST_MESSAGE_QUEUE_URL
TEST_SESSION_STORE_URL
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
ACTIVATION_TOKEN_ISSUER
ACTIVATION_TOKEN_KEY_ID
ACTIVATION_TOKEN_SECRET
AUTHORIZATION_TOKEN_ISSUER
AUTHORIZATION_TOKEN_KEY_ID
AUTHORIZATION_TOKEN_SECRET
APPLICATION_URL
DATABASE_URL
MAILER_DOMAIN
MAILER_FROM_EMAIL
MAILER_FROM_ALIAS
MAILER_SMTP_HOST
MAILER_SMTP_PASSWORD
MAILER_SMTP_USERNAME
MESSAGE_QUEUE_URL
SESSION_STORE_URL
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
TEST_ACTIVATION_TOKEN_ISSUER
TEST_ACTIVATION_TOKEN_KEY_ID
TEST_ACTIVATION_TOKEN_SECRET
TEST_AUTHORIZATION_TOKEN_ISSUER
TEST_AUTHORIZATION_TOKEN_KEY_ID
TEST_AUTHORIZATION_TOKEN_SECRET
TEST_APPLICATION_URL
TEST_DATABASE_URL
TEST_MAILER_DOMAIN
TEST_MAILER_FROM_EMAIL
TEST_MAILER_FROM_ALIAS
TEST_MAILER_SMTP_HOST
TEST_MAILER_SMTP_PASSWORD
TEST_MAILER_SMTP_USERNAME
TEST_MESSAGE_QUEUE_URL
TEST_SESSION_STORE_URL
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
ACTIVATION_TOKEN_ISSUER
ACTIVATION_TOKEN_KEY_ID
ACTIVATION_TOKEN_SECRET
AUTHORIZATION_TOKEN_ISSUER
AUTHORIZATION_TOKEN_KEY_ID
AUTHORIZATION_TOKEN_SECRET
APPLICATION_URL
DATABASE_URL
MAILER_DOMAIN
MAILER_FROM_EMAIL
MAILER_FROM_ALIAS
MAILER_SMTP_HOST
MAILER_SMTP_PASSWORD
MAILER_SMTP_USERNAME
MESSAGE_QUEUE_URL
SESSION_STORE_URL
"#, || {
                let c = Config::from("production").unwrap();
                assert_eq!(c.env_name, "production");
                assert_eq!(c.database_max_pool_size, 12);
                assert_eq!(c.message_queue_max_pool_size, 8);
                assert_eq!(c.session_store_max_pool_size, 8);
            });
        }
    }

    rusty_fork_test! {
        #[test]
        fn test_from_testing() {
            with(r#"
TEST_ACTIVATION_TOKEN_ISSUER
TEST_ACTIVATION_TOKEN_KEY_ID
TEST_ACTIVATION_TOKEN_SECRET
TEST_AUTHORIZATION_TOKEN_ISSUER
TEST_AUTHORIZATION_TOKEN_KEY_ID
TEST_AUTHORIZATION_TOKEN_SECRET
TEST_APPLICATION_URL
TEST_DATABASE_URL
TEST_MAILER_DOMAIN
TEST_MAILER_FROM_EMAIL
TEST_MAILER_FROM_ALIAS
TEST_MAILER_SMTP_HOST
TEST_MAILER_SMTP_PASSWORD
TEST_MAILER_SMTP_USERNAME
TEST_MESSAGE_QUEUE_URL
TEST_SESSION_STORE_URL
"#, || {
                let c = Config::from("testing").unwrap();
                assert_eq!(c.env_name, "testing");
                assert_eq!(c.database_max_pool_size, 2);
                assert_eq!(c.message_queue_max_pool_size, 2);
                assert_eq!(c.session_store_max_pool_size, 2);
            });
        }
    }

    rusty_fork_test! {
        #[test]
        fn test_from_development() {
            with(r#"
ACTIVATION_TOKEN_ISSUER
ACTIVATION_TOKEN_KEY_ID
ACTIVATION_TOKEN_SECRET
AUTHORIZATION_TOKEN_ISSUER
AUTHORIZATION_TOKEN_KEY_ID
AUTHORIZATION_TOKEN_SECRET
APPLICATION_URL
DATABASE_URL
MAILER_DOMAIN
MAILER_FROM_EMAIL
MAILER_FROM_ALIAS
MAILER_SMTP_HOST
MAILER_SMTP_PASSWORD
MAILER_SMTP_USERNAME
MESSAGE_QUEUE_URL
SESSION_STORE_URL
"#, || {
                let c = Config::from("development").unwrap();
                assert_eq!(c.env_name, "development");
                assert_eq!(c.database_max_pool_size, 4);
                assert_eq!(c.message_queue_max_pool_size, 4);
                assert_eq!(c.session_store_max_pool_size, 4);
            });
        }
    }
}
