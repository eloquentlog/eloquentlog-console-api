use std::convert::Into;
use std::fmt;

use diesel::PgConnection;
use slog::Logger;

use config::Config;
use model::user_email::UserEmail;
use model::token::{ActivationClaims, Claims, TokenData};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum JobKind {
    SendUserActivationEmail,
    SendPasswordResetEmail,
}

impl fmt::Display for JobKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Job<T> {
    pub kind: JobKind,
    pub args: Vec<T>,
}

impl<T> Job<T>
where T: fmt::Debug + Copy + Into<i64>
{
    pub fn invoke(
        &self,
        db_conn: &PgConnection,
        logger: &Logger,
        config: &Config,
    )
    {
        match self.kind {
            JobKind::SendUserActivationEmail => {
                self.send_user_activation_email(db_conn, logger, config);
            },
            JobKind::SendPasswordResetEmail => {
                self.send_password_reset_email(db_conn, logger, config);
            },
        }
    }

    fn send_user_activation_email(
        &self,
        db_conn: &PgConnection,
        logger: &Logger,
        config: &Config,
    )
    {
        let args = self.args.as_slice();
        if args.is_empty() {
            return;
        }

        let id = args[0].into();
        match UserEmail::find_by_id(id, db_conn, &logger) {
            Some(ref email) => {
                info!(
                    logger,
                    "user_email.email: {}",
                    email.email.as_ref().unwrap()
                );

                // TODO: implement UserEmail into TokenData
                let data = TokenData {
                    value: email.activation_token.as_ref().unwrap().to_string(),
                    granted_at: email
                        .activation_token_granted_at
                        .unwrap()
                        .timestamp(),
                    expires_at: email
                        .activation_token_expires_at
                        .unwrap()
                        .timestamp(),
                };

                let token = ActivationClaims::encode(
                    data,
                    &config.activation_token_issuer,
                    &config.activation_token_key_id,
                    &config.activation_token_secret,
                );

                // TODO
                dbg!(token);
            },
            _ => {
                error!(logger, "not found :'(");
            },
        }
    }

    fn send_password_reset_email(
        &self,
        _: &PgConnection,
        logger: &Logger,
        _: &Config,
    )
    {
        // TODO
        info!(logger, "args: {:#?}", self.args.as_slice());
    }
}
