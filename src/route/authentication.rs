use chrono::Utc;
use rocket::State;
use rocket::http::Status;
use rocket::response::Response as RawResponse;
use rocket_slog::SyncLogger;

use config::Config;
use db::DbConn;
use model::user::User;
use model::ticket::{AuthorizationClaims, Claims, Token};
use request::user::UserSignIn as RequestData;
use response::{Response, no_content_for};

#[options("/login")]
pub fn login_options<'a>() -> RawResponse<'a> {
    no_content_for("POST")
}

#[post("/login", data = "<data>", format = "json")]
pub fn login(
    conn: DbConn,
    data: RequestData,
    logger: SyncLogger,
    config: State<Config>,
) -> Response
{
    let res: Response = Default::default();

    match User::find_by_email_or_uuid(&data.username, &conn, &logger) {
        Some(ref user) if user.verify_password(&data.password) => {
            // TODO
            // set valid expires_at and impl review mechanism (check also
            // `validate_exp` for Validation struct for JWT)
            // e.g. let expires_at = (now + Duration::weeks(2)).timestamp();
            let token = Token {
                value: user.uuid.to_urn().to_string(),
                granted_at: Utc::now().timestamp(),
                expires_at: 0,
            };
            let ticket = AuthorizationClaims::encode(
                token,
                &config.authorization_ticket_issuer,
                &config.authorization_ticket_key_id,
                &config.authorization_ticket_secret,
            );
            res.format(json!({"ticket": ticket.to_string()}))
        },
        _ => {
            warn!(logger, "login failed: username {}", data.username);

            res.status(Status::Unauthorized).format(json!({
                "message": "The credentials you've entered are incorrect"
            }))
        },
    }
}

#[post("/logout", format = "json")]
pub fn logout(user: &User, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    // TODO
    info!(logger, "user: {}", user.uuid);

    res.status(Status::UnprocessableEntity)
}
