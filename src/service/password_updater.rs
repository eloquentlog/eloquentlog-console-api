use std::fmt;

use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::{Authenticatable, Verifiable};

pub struct PasswordUpdater<'a, T>
where T: Authenticatable + Clone + Verifiable<T> + fmt::Display
{
    db_conn: &'a DbConn,
    config: &'a Config,
    logger: &'a SyncLogger,
    pub target: Option<T>,
}

impl<'a, T> PasswordUpdater<'a, T>
where T: Authenticatable + Clone + Verifiable<T> + fmt::Display
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

    fn load_target(&self, token: &str) -> Result<T, &str> {
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

    pub fn load(mut self, token: &str) -> Result<Self, &str> {
        self.target = self.load_target(token).ok();
        Ok(self)
    }

    pub fn update(&self, new_password: &str) -> Result<(), &str> {
        if let Some(mut user) = self.target.clone() {
            return user
                .update_password(new_password, self.db_conn, self.logger)
                .map(|_| {
                    info!(
                        self.logger,
                        "the password of an user ({}) has been re-set", &user
                    );
                })
                .map_err(|e| {
                    warn!(
                        self.logger,
                        "the password of an user ({}) couldn't be set", &user
                    );
                    e
                });
        }
        Err("not found")
    }
}
