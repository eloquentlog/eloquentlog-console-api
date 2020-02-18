use chrono::Utc;
use diesel::result::Error;
use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::access_token::{AccessToken, AccessTokenState, AgentType};
use crate::model::token::{AuthenticationClaims, Claims, TokenData};
use crate::model::user::User;
use crate::response::Response;

pub mod preflight {
    use rocket::http::Status;
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;

    use crate::model::access_token::AgentType;
    use crate::response::no_content_for;

    #[options("/access_token/lpop/<key>")]
    pub fn lpop<'a>(key: AgentType) -> RawResponse<'a> {
        if key == AgentType::Person {
            return no_content_for("PATCH");
        }

        let mut res = RawResponse::new();
        res.set_status(Status::NotFound);
        res
    }

    #[options("/access_token/lrange/<key>/<start>/<stop>")]
    pub fn lrange<'a>(
        key: AgentType,
        start: i64,
        stop: i64,
        logger: SyncLogger,
    ) -> RawResponse<'a>
    {
        if key == AgentType::Person {
            info!(logger, "start: {}", start);
            info!(logger, "stop: {}", stop);

            return no_content_for("GET");
        }

        let mut res = RawResponse::new();
        res.set_status(Status::NotFound);
        res
    }
}

// lpop returns a personal token of the user.
// This function actually generates new token, and the value is currently
// not saved in datadase. User can call only once.
#[patch("/access_token/lpop/<key>")]
pub fn lpop<'a>(
    key: AgentType,
    user: &User,
    conn: DbConn,
    config: State<Config>,
    logger: SyncLogger,
) -> Response<'a>
{
    info!(logger, "user: {}", user.uuid);

    let res: Response = Default::default();

    // only personal token
    if key != AgentType::Person {
        return res.status(Status::NotFound);
    }

    let result: Result<AccessToken, Error> = conn
        .build_transaction()
        .serializable()
        .deferrable()
        .read_write()
        .run::<AccessToken, diesel::result::Error, _>(|| {
            match AccessToken::find_disabled_personal_token_by_user_id(
                user.id, &conn, &logger,
            ) {
                None => {
                    error!(logger, "err: not found user.id {}", user.uuid);
                    Err(Error::RollbackTransaction)
                },
                Some(mut t) => {
                    match t.mark_as(AccessTokenState::Enabled, &conn, &logger) {
                        Err(e) => {
                            error!(logger, "err: {}", e);
                            Err(Error::RollbackTransaction)
                        },
                        Ok(_) => {
                            let data = TokenData {
                                value: String::from_utf8(t.token.unwrap())
                                    .unwrap(),
                                granted_at: Utc::now().timestamp(),
                                expires_at: 0,
                            };

                            // this value is not saved
                            let token = AuthenticationClaims::encode(
                                data,
                                &config.authentication_token_issuer,
                                &config.authentication_token_key_id,
                                &config.authentication_token_secret,
                            );
                            t.token = Some(token.into_bytes());

                            Ok(t)
                        },
                    }
                },
            }
        });
    if result.is_err() {
        return res.status(Status::NotFound);
    }

    // FIXME: refactor
    let t = result.unwrap();
    let token = String::from_utf8(t.token.unwrap()).unwrap();
    res.format(json!({
        "access_token": {
            "name": t.name,
            "token": token,
            "created_at": t.created_at,
            "updated_at": t.updated_at,
        }
    }))
}

#[get("/access_token/lrange/<key>/<start>/<stop>")]
pub fn lrange<'a>(
    key: AgentType,
    start: i64,
    stop: i64,
    user: &User,
    conn: DbConn,
    logger: SyncLogger,
) -> Response<'a>
{
    let res: Response = Default::default();

    info!(logger, "user: {}", user.uuid);
    info!(logger, "key: {}, start: {}, stop: {}", key, start, stop);

    if key != AgentType::Client {
        return res.format(json!([]));
    }

    // TODO
    let mut offset = start;
    if offset < 1 {
        offset = 0;
    }

    let mut limit = stop - start + 2;
    if limit < 1 {
        limit = 1;
    }

    let data = match AccessToken::fetch_enabled_client_tokens_by_user_id(
        user.id, offset, limit, &conn, &logger,
    ) {
        None => {
            error!(logger, "err: not found user.id {}", user.uuid);
            vec![]
        },
        Some(a) => {
            a.iter()
                .map(|t| {
                    json!({
                        "access_token": {
                            "name": t.name,
                            "token": "...",
                            "created_at": t.created_at,
                            "updated_at": t.updated_at,
                        }
                    })
                })
                .collect()
        },
    };
    res.format(json!(data))
}
