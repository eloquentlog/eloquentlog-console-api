use std::fmt;

use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::{Activatable, Verifiable};

pub struct AccountActivator<'a, T, U>
where
    T: Activatable + Clone + Verifiable<(T, U)> + fmt::Display,
    U: Activatable + Clone + fmt::Display,
{
    db_conn: &'a DbConn,
    config: &'a Config,
    logger: &'a SyncLogger,
    pub target: Option<(T, U)>,
}

impl<'a, T, U> AccountActivator<'a, T, U>
where
    T: Activatable + Clone + Verifiable<(T, U)> + fmt::Display,
    U: Activatable + Clone + fmt::Display,
{
    pub fn new(
        db_conn: &'a DbConn,
        config: &'a Config,
        logger: &'a SyncLogger,
    ) -> Self
    {
        Self {
            db_conn,
            config,
            logger,
            target: None,
        }
    }

    fn load_target(&self, token: &'a str) -> Result<(T, U), &'a str> {
        let concrete_token = T::extract_concrete_token(
            token,
            &self.config.verification_token_issuer,
            &self.config.verification_token_secret,
        )?;
        T::load_by_concrete_token(&concrete_token, self.db_conn, self.logger)
            .map_err(|e| {
                warn!(self.logger, "err: {}", e);
                "not found"
            })
    }

    pub fn load(mut self, token: &'a str) -> Result<Self, &'a str> {
        self.target = self.load_target(token).ok();
        Ok(self)
    }

    pub fn activate(&self) -> Result<(), &str> {
        if let Some(user) = self.target.as_ref().map(|v| v.0.clone()) {
            return user
                .activate(&self.db_conn, &self.logger)
                .map(|_| {
                    info!(
                        self.logger,
                        "the user ({}) has been activated", &user
                    );
                })
                .map_err(|e| {
                    warn!(
                        self.logger,
                        "the user ({}) couldn't be activated", &user
                    );
                    e
                });
        }
        Err("not found")
    }
}
