use diesel::result::Error;
use rocket::State;
use rocket::http::Status;
use rocket_slog::SyncLogger;
use serde_json::Value;

use crate::config::Config;
use crate::db::DbConn;
use crate::model::access_token::{AccessToken, AccessTokenState, AgentType};
use crate::model::token::{AuthenticationClaims, Claims, TokenData};
use crate::model::user::User;
use crate::response::Response;

pub mod preflight {
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;

    use crate::model::access_token::{AgentType, AccessTokenState};
    use crate::response::no_content_for;

    #[options("/access_token/del/<uuid>")]
    pub fn del<'a>(uuid: String, logger: SyncLogger) -> RawResponse<'a> {
        info!(logger, "uuid: {}", uuid);
        no_content_for("PATCH")
    }

    #[options("/access_token/dump/<uuid>")]
    pub fn dump<'a>(uuid: String, logger: SyncLogger) -> RawResponse<'a> {
        info!(logger, "uuid: {}", uuid);
        no_content_for("PATCH")
    }

    #[options("/access_token/hset/<uuid>/state/<access_token_state>")]
    pub fn hset_state<'a>(
        uuid: String,
        access_token_state: AccessTokenState,
        logger: SyncLogger,
    ) -> RawResponse<'a>
    {
        info!(logger, "uuid: {}, state: {}", uuid, access_token_state);
        no_content_for("PATCH")
    }

    #[options("/access_token/append/<agent_type>")]
    pub fn append<'a>(
        agent_type: AgentType,
        logger: SyncLogger,
    ) -> RawResponse<'a>
    {
        info!(logger, "agent_type: {}", agent_type);
        no_content_for("PUT")
    }

    #[options("/access_token/lrange/<agent_type>/<start>/<stop>")]
    pub fn lrange<'a>(
        agent_type: AgentType,
        start: i64,
        stop: i64,
        logger: SyncLogger,
    ) -> RawResponse<'a>
    {
        info!(
            logger,
            "agent_type: {}, start: {}, stop: {}", agent_type, start, stop,
        );
        no_content_for("GET")
    }
}

#[patch("/access_token/dump/<uuid>")]
pub fn dump<'a>(
    uuid: String,
    user: &User,
    conn: DbConn,
    config: State<Config>,
    logger: SyncLogger,
) -> Response<'a>
{
    info!(logger, "user: {}, uuid: {}", user.uuid, uuid);

    let res: Response = Default::default();

    let result: Result<AccessToken, Error> = conn
        .build_transaction()
        .serializable()
        .deferrable()
        .read_write()
        .run::<AccessToken, diesel::result::Error, _>(|| {
            match AccessToken::owned_by_uuid(&user, &uuid, &conn, &logger) {
                // Note: this is available only once
                Some(mut t) if t.token.is_none() => {
                    let token = AccessToken::generate_token();
                    match t.update_token(&token, &conn, &logger) {
                        Err(e) => {
                            error!(logger, "err: {}", e);
                            Err(Error::RollbackTransaction)
                        },
                        Ok(a) => {
                            let value =
                                String::from_utf8(a.token.unwrap()).unwrap();

                            // TODO: set expires_at
                            let data = TokenData {
                                value,
                                granted_at: a.updated_at.timestamp(),
                                expires_at: 0,
                            };
                            let value = AuthenticationClaims::encode(
                                data,
                                &config.authentication_token_issuer,
                                &config.authentication_token_key_id,
                                &config.authentication_token_secret,
                            );
                            t.token = Some(value.into_bytes());
                            Ok(t)
                        },
                    }
                },
                _ => {
                    error!(logger, "err: not found {}", uuid);
                    Err(Error::RollbackTransaction)
                },
            }
        });

    if result.is_err() {
        return res.status(Status::NotFound);
    }

    let t = result.unwrap();
    let token = String::from_utf8(t.token.unwrap()).unwrap();
    res.format(json!({
        "access_token": {
            "uuid": t.uuid.to_string(),
            "name": t.name,
            "agent_type": t.agent_type.to_string(),
            "state": t.state.to_string(),
            "token": token,
            "revoked_at": Value::Null,
            "created_at": t.created_at,
            "updated_at": t.updated_at,
        }
    }))
}

#[patch("/access_token/del/<uuid>")]
pub fn del<'a>(
    uuid: String,
    user: &User,
    conn: DbConn,
    logger: SyncLogger,
) -> Response<'a>
{
    info!(logger, "user: {}, uuid: {}", user.uuid, uuid);

    let res: Response = Default::default();

    let result: Result<(), Error> = conn
        .build_transaction()
        .serializable()
        .deferrable()
        .read_write()
        .run::<(), diesel::result::Error, _>(|| {
            match AccessToken::owned_by_uuid(&user, &uuid, &conn, &logger) {
                None => {
                    error!(logger, "err: not found {}", uuid);
                    Err(Error::RollbackTransaction)
                },
                Some(t) => {
                    match t.revoke(&conn, &logger) {
                        Err(e) => {
                            error!(logger, "err: {}", e);
                            Err(Error::RollbackTransaction)
                        },
                        Ok(_) => Ok(()),
                    }
                },
            }
        });

    if result.is_err() {
        return res.status(Status::NotFound);
    }

    res.format(json!({"access_token": 1}))
}

#[patch("/access_token/hset/<uuid>/state/<access_token_state>")]
pub fn hset_state<'a>(
    uuid: String,
    access_token_state: AccessTokenState,
    user: &User,
    conn: DbConn,
    logger: SyncLogger,
) -> Response<'a>
{
    info!(logger, "user: {}, uuid: {}", user.uuid, uuid);

    let res: Response = Default::default();

    let result: Result<(), Error> = conn
        .build_transaction()
        .serializable()
        .deferrable()
        .read_write()
        .run::<(), diesel::result::Error, _>(|| {
            match AccessToken::owned_by_uuid(&user, &uuid, &conn, &logger) {
                None => {
                    error!(logger, "err: not found {}", uuid);
                    Err(Error::RollbackTransaction)
                },
                Some(t) => {
                    // this does not check old state
                    match t.mark_as(access_token_state, &conn, &logger) {
                        Err(e) => {
                            error!(logger, "err: {}", e);
                            Err(Error::RollbackTransaction)
                        },
                        Ok(_) => Ok(()),
                    }
                },
            }
        });

    if result.is_err() {
        return res.status(Status::NotFound);
    }

    res.format(json!({"access_token": 1}))
}

#[put("/access_token/append/<agent_type>")]
pub fn append<'a>(
    user: &User,
    agent_type: AgentType,
    logger: SyncLogger,
) -> Response<'a>
{
    info!(logger, "user: {}, agent_type: {}", user.uuid, agent_type);

    // TODO
    let res: Response = Default::default();
    res
}

#[get("/access_token/lrange/<agent_type>/<start>/<stop>")]
pub fn lrange<'a>(
    agent_type: AgentType,
    start: i64,
    stop: i64,
    user: &User,
    conn: DbConn,
    logger: SyncLogger,
) -> Response<'a>
{
    info!(
        logger,
        "user: {}, agent_type: {}, start: {}, stop: {}",
        user.uuid,
        agent_type,
        start,
        stop,
    );

    let res: Response = Default::default();

    // TODO
    let mut offset = start;
    if offset < 1 {
        offset = 0;
    }

    let mut limit = stop - start + 2;
    if limit < 1 {
        limit = 1;
    }

    let data = match AccessToken::owned_all_by_agent_type(
        &user, agent_type, offset, limit, &conn, &logger,
    ) {
        None => {
            error!(logger, "err: not found user.id {}", user.uuid);
            vec![]
        },
        Some(a) => {
            let token = "***";
            a.iter()
                .map(|t| {
                    json!({
                        "access_token": {
                            "uuid": t.uuid.to_string(),
                            "name": t.name,
                            "agent_type": t.agent_type.to_string(),
                            "state": t.state.to_string(),
                            "token": token,
                            "revoked_at": Value::Null,
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
