pub mod authentication;
pub mod registration;

use rocket::{Request, State, request};
use rocket::request::FromRequest;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::token::AuthenticationClaims;
use crate::model::user::User;
use crate::request::token::authentication::AuthenticationToken;

/// User
impl<'a, 'r> FromRequest<'a, 'r> for &'a User {
    type Error = ();

    fn from_request(req: &'a Request<'r>) -> request::Outcome<&'a User, ()> {
        let login = req.local_cache(|| {
            let config = req.guard::<State<Config>>().unwrap();
            let db_conn = req.guard::<DbConn>().unwrap();
            let logger = req.guard::<SyncLogger>().unwrap();

            let authentication_token =
                req.local_cache(|| req.guard::<AuthenticationToken>().unwrap());

            User::find_by_token::<AuthenticationClaims>(
                &authentication_token,
                &config.authentication_token_issuer,
                &config.authentication_token_secret,
                &db_conn,
                &logger,
            )
        });
        if let Some(ref user) = login {
            return request::Outcome::Success(user);
        }
        request::Outcome::Forward(())
    }
}
