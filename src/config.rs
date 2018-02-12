use std::env;


pub struct Config {
    pub database_url: String,
    pub env_name: &'static str,
}


impl Default for Config {
    fn default() -> Config {
        Config {
            database_url: env::var("DATABASE_URL").expect(
                "DATABASE_URL is not set"),
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
            env_name: &"publication",
            ..Default::default()
        }
    }

    fn testing_config() -> Config {
        Config {
            database_url: env::var("TEST_DATABASE_URL").expect(
                "TEST_DATABASE_URL is not set"),
            env_name: &"testing",
            ..Default::default()
        }
    }

    fn development_config() -> Config {
        Config {
            env_name: &"development",
            ..Default::default()
        }
    }
}
