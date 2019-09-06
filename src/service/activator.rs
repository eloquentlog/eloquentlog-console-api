use diesel::result::Error;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::token::{ActivationClaims, Claims};
use model::user::User;

pub struct UserActivator<'a> {
    conn: &'a DbConn,
    config: &'a Config,
    logger: &'a SyncLogger,
}

impl<'a> UserActivator<'a> {
    pub fn new(
        conn: &'a DbConn,
        config: &'a Config,
        logger: &'a SyncLogger,
    ) -> Self
    {
        Self {
            conn,
            config,
            logger,
        }
    }

    fn decode_token(&self, token: &str) -> Result<String, &str> {
        let claims = ActivationClaims::decode(
            token,
            &self.config.activation_token_issuer,
            &self.config.activation_token_secret,
        )
        .map_err(|e| {
            warn!(self.logger, "decoding failed: {}", e);
            "invalid token"
        })?;
        Ok(claims.get_subject())
    }

    /// activate finds user account by the activation token, and activates it.
    pub fn activate(&self, token: &str) -> Result<(), &str> {
        let activation_token = self.decode_token(token)?;

        let (user, user_email) = User::load_with_user_email_activation_token(
            &activation_token,
            self.conn,
            self.logger,
        )
        .map_err(|e| {
            warn!(self.logger, "err: {}", e);
            "not found"
        })?;

        let activation = self
            .conn
            .build_transaction()
            .serializable()
            .deferrable()
            .read_write()
            .run::<_, diesel::result::Error, _>(|| {
                if user_email.is_primary() &&
                    user_email.activate(&self.conn, &self.logger).is_ok() &&
                    user.activate(&self.conn, &self.logger).is_ok()
                {
                    return Ok(());
                }
                Err(Error::RollbackTransaction)
            });
        if activation.is_ok() {
            info!(
                self.logger,
                "an user ({}) has been activated (granted: {})",
                user,
                user_email
                    .activation_token_granted_at
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S"),
            );
            return Ok(());
        }
        warn!(self.logger, "an user ({}) could not be activated", user);
        Err("activation failed")
    }
}
