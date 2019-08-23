use std::convert::Into;
use std::fmt;

use diesel::PgConnection;
use diesel::result::Error;
use slog::Logger;

use config::Config;
use model::token::{ActivationClaims, Claims};
use model::user::User;
use model::user_email::UserEmail;
use mailer::user::UserMailer;

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
        config: &Config,
        logger: &Logger,
    )
    {
        match self.kind {
            JobKind::SendUserActivationEmail => {
                self.send_user_activation_email(db_conn, config, logger);
            },
            JobKind::SendPasswordResetEmail => {
                self.send_password_reset_email(db_conn, config, logger);
            },
        }
    }

    fn send_user_activation_email(
        &self,
        db_conn: &PgConnection,
        config: &Config,
        logger: &Logger,
    )
    {
        let args = self.args.as_slice();
        if args.is_empty() {
            return;
        }
        let id = args[0].into();
        let _: Result<_, Error> = db_conn
            .build_transaction()
            .read_only()
            .run::<_, diesel::result::Error, _>(|| {
            match UserEmail::find_by_id(id, db_conn, &logger) {
                Some(ref user_email) => {
                    let email = user_email.email.as_ref().unwrap();
                    info!(logger, "user_email.email: {}", email);

                    let token = ActivationClaims::encode(
                        user_email.into(),
                        &config.activation_token_issuer,
                        &config.activation_token_key_id,
                        &config.activation_token_secret,
                    );
                    info!(logger, "token: {}", token);

                    // TODO: replace it find_by_id (or where primary)
                    let user =
                        User::find_by_email(email, db_conn, logger).unwrap();

                    let mut mailer = UserMailer::new(config, logger);
                    let user_name = Box::leak(
                        user.name
                            .unwrap_or_else(|| "".to_string())
                            .into_boxed_str(),
                    );
                    mailer
                        .to((email, user_name))
                        .send_user_activation_email(token);
                    Ok(())
                },
                _ => {
                    error!(logger, "not found :'(");
                    Err(Error::RollbackTransaction)
                },
            }
        });
    }

    fn send_password_reset_email(
        &self,
        _: &PgConnection,
        _: &Config,
        logger: &Logger,
    )
    {
        // TODO
        info!(logger, "args: {:#?}", self.args.as_slice());
    }
}
