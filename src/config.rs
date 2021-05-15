use std::env;

#[derive(Clone)]
pub struct Config {
    pub application_url: String,
    pub authentication_token_issuer: String,
    pub authentication_token_key_id: String,
    pub authentication_token_secret: String,
    pub cookie_domain: String,
    pub cookie_secure: bool,
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
    pub verification_token_issuer: String,
    pub verification_token_key_id: String,
    pub verification_token_secret: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            application_url: env::var("APPLICATION_URL")
                .expect("APPLICATION_URL is not set"),

            authentication_token_issuer: env::var(
                "AUTHENTICATION_TOKEN_ISSUER",
            )
            .expect("AUTHENTICATION_TOKEN_ISSUER is not set"),
            authentication_token_key_id: env::var(
                "AUTHENTICATION_TOKEN_KEY_ID",
            )
            .expect("AUTHENTICATION_TOKEN_KEY_ID is not set"),
            authentication_token_secret: env::var(
                "AUTHENTICATION_TOKEN_SECRET",
            )
            .expect("AUTHENTICATION_TOKEN_SECRET is not set"),

            cookie_domain: env::var("COOKIE_DOMAIN")
                .expect("COOKIE_DOMAIN is not set"),
            cookie_secure: env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| "false".to_string()) ==
                "true",

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

            verification_token_issuer: env::var("VERIFICATION_TOKEN_ISSUER")
                .expect("VERIFICATION_TOKEN_ISSUER is not set"),
            verification_token_key_id: env::var("VERIFICATION_TOKEN_KEY_ID")
                .expect("VERIFICATION_TOKEN_KEY_ID is not set"),
            verification_token_secret: env::var("VERIFICATION_TOKEN_SECRET")
                .expect("VERIFICATION_TOKEN_SECRET is not set"),
        }
    }
}

impl Config {
    pub const CSRF_HASH_DURATION: i64 = 10; // minutes
    pub const CSRF_HASH_LENGTH: i32 = 32;
    pub const CSRF_HASH_SOURCE: &'static [u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz01234567890-_";

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
            cookie_secure: true,
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
            application_url: env::var("TEST_APPLICATION_URL")
                .expect("TEST_APPLICATION_URL is not set"),

            authentication_token_issuer: env::var(
                "TEST_AUTHENTICATION_TOKEN_ISSUER",
            )
            .expect("TEST_AUTHENTICATION_TOKEN_ISSUER is not set"),
            authentication_token_key_id: env::var(
                "TEST_AUTHENTICATION_TOKEN_KEY_ID",
            )
            .expect("TEST_AUTHENTICATION_TOKEN_KEY_ID is not set"),
            authentication_token_secret: env::var(
                "TEST_AUTHENTICATION_TOKEN_SECRET",
            )
            .expect("TEST_AUTHENTICATION_TOKEN_SECRET is not set"),

            cookie_domain: env::var("TEST_COOKIE_DOMAIN")
                .expect("TEST_COOKIE_DOMAIN is not set"),
            cookie_secure: env::var("TEST_COOKIE_SECURE")
                .unwrap_or_else(|_| "false".to_string()) ==
                "true",

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

            verification_token_issuer: env::var(
                "TEST_VERIFICATION_TOKEN_ISSUER",
            )
            .expect("TEST_VERIFICATION_TOKEN_ISSUER is not set"),
            verification_token_key_id: env::var(
                "TEST_VERIFICATION_TOKEN_KEY_ID",
            )
            .expect("TEST_VERIFICATION_TOKEN_KEY_ID is not set"),
            verification_token_secret: env::var(
                "TEST_VERIFICATION_TOKEN_SECRET",
            )
            .expect("TEST_VERIFICATION_TOKEN_SECRET is not set"),
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

    use crate::hashmap;

    // TODO: set HashMap as an arg
    fn with<T>(keys: &'static str, test: T)
    where T: FnOnce() + panic::UnwindSafe {
        lazy_static! {
            static ref ENV_LOCK: Mutex<()> = Mutex::new(());
            static ref TESTS: HashMap<&'static str, &'static str> = hashmap! {
                "APPLICATION_URL" => "http://127.0.0.1:3000",
                "AUTHENTICATION_TOKEN_ISSUER" => "com.eloquentlog",
                "AUTHENTICATION_TOKEN_KEY_ID" => "key_id-authentication",
                "AUTHENTICATION_TOKEN_SECRET" => "secret-authentication",
                "COOKIE_DOMAIN" => "127.0.0.1",
                "COOKIE_SECURE" => "false",
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
                "VERIFICATION_TOKEN_ISSUER" => "com.eloquentlog",
                "VERIFICATION_TOKEN_KEY_ID" => "key_id-verification",
                "VERIFICATION_TOKEN_SECRET" => "secret-verification",

                "TEST_APPLICATION_URL" => "http://127.0.0.1:3000",
                "TEST_AUTHENTICATION_TOKEN_ISSUER" => "com.eloquentlog",
                "TEST_AUTHENTICATION_TOKEN_KEY_ID" => "test-key_id-authentication",
                "TEST_AUTHENTICATION_TOKEN_SECRET" => "test-secret-authentication",
                "TEST_COOKIE_DOMAIN" => "127.0.0.1",
                "TEST_COOKIE_SECURE" => "false",
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
                    "redis://u$er:pa$$w0rd@localhost:6379/session",
                "TEST_VERIFICATION_TOKEN_ISSUER" => "com.eloquentlog",
                "TEST_VERIFICATION_TOKEN_KEY_ID" => "test-key_id-verification",
                "TEST_VERIFICATION_TOKEN_SECRET" => "test-secret-verification"
            };
        }

        let _lock = ENV_LOCK.lock();

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut origins: HashMap<&str, Result<String, env::VarError>> =
                HashMap::new();

            for (key, var) in TESTS.iter() {
                origins.insert(key, env::var(key));

                if !keys.split('\n').any(|x| &x == key) {
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
TEST_APPLICATION_URL
TEST_AUTHENTICATION_TOKEN_ISSUER
TEST_AUTHENTICATION_TOKEN_KEY_ID
TEST_AUTHENTICATION_TOKEN_SECRET
TEST_COOKIE_DOMAIN
TEST_COOKIE_SECURE
TEST_DATABASE_URL
TEST_MAILER_DOMAIN
TEST_MAILER_FROM_EMAIL
TEST_MAILER_FROM_ALIAS
TEST_MAILER_SMTP_HOST
TEST_MAILER_SMTP_PASSWORD
TEST_MAILER_SMTP_USERNAME
TEST_MESSAGE_QUEUE_URL
TEST_SESSION_STORE_URL
TEST_VERIFICATION_TOKEN_ISSUER
TEST_VERIFICATION_TOKEN_KEY_ID
TEST_VERIFICATION_TOKEN_SECRET
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
APPLICATION_URL
AUTHENTICATION_TOKEN_ISSUER
AUTHENTICATION_TOKEN_KEY_ID
AUTHENTICATION_TOKEN_SECRET
COOKIE_DOMAIN
COOKIE_SECURE
DATABASE_URL
MAILER_DOMAIN
MAILER_FROM_EMAIL
MAILER_FROM_ALIAS
MAILER_SMTP_HOST
MAILER_SMTP_PASSWORD
MAILER_SMTP_USERNAME
MESSAGE_QUEUE_URL
SESSION_STORE_URL
VERIFICATION_TOKEN_ISSUER
VERIFICATION_TOKEN_KEY_ID
VERIFICATION_TOKEN_SECRET
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
TEST_APPLICATION_URL
TEST_AUTHENTICATION_TOKEN_ISSUER
TEST_AUTHENTICATION_TOKEN_KEY_ID
TEST_AUTHENTICATION_TOKEN_SECRET
TEST_COOKIE_DOMAIN
TEST_COOKIE_SECURE
TEST_DATABASE_URL
TEST_MAILER_DOMAIN
TEST_MAILER_FROM_EMAIL
TEST_MAILER_FROM_ALIAS
TEST_MAILER_SMTP_HOST
TEST_MAILER_SMTP_PASSWORD
TEST_MAILER_SMTP_USERNAME
TEST_MESSAGE_QUEUE_URL
TEST_SESSION_STORE_URL
TEST_VERIFICATION_TOKEN_ISSUER
TEST_VERIFICATION_TOKEN_KEY_ID
TEST_VERIFICATION_TOKEN_SECRET
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
APPLICATION_URL
AUTHENTICATION_TOKEN_ISSUER
AUTHENTICATION_TOKEN_KEY_ID
AUTHENTICATION_TOKEN_SECRET
COOKIE_DOMAIN
COOKIE_SECURE
DATABASE_URL
MAILER_DOMAIN
MAILER_FROM_EMAIL
MAILER_FROM_ALIAS
MAILER_SMTP_HOST
MAILER_SMTP_PASSWORD
MAILER_SMTP_USERNAME
MESSAGE_QUEUE_URL
SESSION_STORE_URL
VERIFICATION_TOKEN_ISSUER
VERIFICATION_TOKEN_KEY_ID
VERIFICATION_TOKEN_SECRET
"#, || {
                let c = Config::from("production").unwrap();
                assert_eq!(c.env_name, "production");
                assert!(c.cookie_secure);
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
TEST_APPLICATION_URL
TEST_AUTHENTICATION_TOKEN_ISSUER
TEST_AUTHENTICATION_TOKEN_KEY_ID
TEST_AUTHENTICATION_TOKEN_SECRET
TEST_COOKIE_DOMAIN
TEST_COOKIE_SECURE
TEST_DATABASE_URL
TEST_MAILER_DOMAIN
TEST_MAILER_FROM_EMAIL
TEST_MAILER_FROM_ALIAS
TEST_MAILER_SMTP_HOST
TEST_MAILER_SMTP_PASSWORD
TEST_MAILER_SMTP_USERNAME
TEST_MESSAGE_QUEUE_URL
TEST_SESSION_STORE_URL
TEST_VERIFICATION_TOKEN_ISSUER
TEST_VERIFICATION_TOKEN_KEY_ID
TEST_VERIFICATION_TOKEN_SECRET
"#, || {
                let c = Config::from("testing").unwrap();
                assert_eq!(c.env_name, "testing");
                assert!(c.cookie_secure);
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
APPLICATION_URL
AUTHENTICATION_TOKEN_ISSUER
AUTHENTICATION_TOKEN_KEY_ID
AUTHENTICATION_TOKEN_SECRET
COOKIE_DOMAIN
COOKIE_SECURE
DATABASE_URL
MAILER_DOMAIN
MAILER_FROM_EMAIL
MAILER_FROM_ALIAS
MAILER_SMTP_HOST
MAILER_SMTP_PASSWORD
MAILER_SMTP_USERNAME
MESSAGE_QUEUE_URL
SESSION_STORE_URL
VERIFICATION_TOKEN_ISSUER
VERIFICATION_TOKEN_KEY_ID
VERIFICATION_TOKEN_SECRET
"#, || {
                let c = Config::from("development").unwrap();
                assert_eq!(c.env_name, "development");
                assert!(c.cookie_secure);
                assert_eq!(c.database_max_pool_size, 4);
                assert_eq!(c.message_queue_max_pool_size, 4);
                assert_eq!(c.session_store_max_pool_size, 4);
            });
        }
    }
}
