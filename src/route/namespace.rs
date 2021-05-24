use diesel::result::Error;
use rocket::http::Status;
use rocket_contrib::json::{Json, JsonValue};
use rocket_slog::SyncLogger;

use crate::db::DbConn;
use crate::model::namespace::{Namespace, NewNamespace};
use crate::model::user::User;
use crate::model::membership::{Membership, MembershipRole, NewMembership};
use crate::response::Response;
use crate::request::namespace::Namespace as RequestData;
use crate::validation::namespace::Validator;

pub mod preflight {
    use rocket::State;
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;

    use crate::config::Config;
    use crate::response::no_content_for;

    #[options("/namespace/hget/<uuid>")]
    pub fn hget<'a>(
        uuid: String,
        config: State<Config>,
        logger: SyncLogger,
    ) -> RawResponse<'a> {
        info!(logger, "hget uuid: {}", uuid);
        no_content_for("GET", &config)
    }

    #[options("/namespace/hgetall")]
    pub fn hgetall<'a>(
        config: State<Config>,
        logger: SyncLogger,
    ) -> RawResponse<'a> {
        info!(logger, "hgetall");
        no_content_for("GET", &config)
    }

    #[options("/namespace/hset")]
    pub fn hset<'a>(
        config: State<Config>,
        logger: SyncLogger,
    ) -> RawResponse<'a> {
        info!(logger, "hset");
        no_content_for("GET", &config)
    }
}

#[get("/namespace/hget/<uuid>")]
pub fn hget(
    uuid: String,
    user: &User,
    conn: DbConn,
    logger: SyncLogger,
) -> Response {
    info!(logger, "user: {}, uuid: {}", user.uuid, uuid);

    let res: Response = Default::default();

    let data: Result<JsonValue, Error> =
        match Namespace::find_by_uuid(&uuid, &user, &conn, &logger) {
            None => {
                error!(logger, "err: no namespace for uuid: {}", uuid);
                Err(Error::NotFound)
            },
            Some(n) => Ok(json!({ "namespace": n })),
        };
    if data.is_err() {
        return res.status(Status::NotFound);
    }
    res.format(data.unwrap())
}

#[get("/namespace/hgetall")]
pub fn hgetall(user: &User, conn: DbConn, logger: SyncLogger) -> Response {
    let res: Response = Default::default();

    info!(logger, "user: {}", user.uuid);

    let data = match Namespace::find_all(user, &conn, &logger) {
        None => {
            error!(logger, "err: no namespace for user: {}", user.uuid);
            vec![]
        },
        Some(a) => a.iter().map(|n| json!({ "namespace": n })).collect(),
    };
    res.format(json!(data))
}

#[post("/namespace/hset", format = "json", data = "<data>")]
pub fn hset(
    user: &User,
    data: Json<RequestData>,
    conn: DbConn,
    logger: SyncLogger,
) -> Response {
    let res: Response = Default::default();

    info!(logger, "user: {}", user.uuid);

    let v = Validator::new(&data, &logger);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            let result: Result<String, Error> = conn
                .build_transaction()
                .serializable()
                .deferrable()
                .read_write()
                .run::<String, diesel::result::Error, _>(|| {
                    let n = NewNamespace::from(data.0.clone());
                    if let Some(namespace) =
                        Namespace::insert(&n, &conn, &logger)
                    {
                        info!(logger, "namespace: {}", namespace.id);
                        let m = NewMembership {
                            namespace_id: namespace.id,
                            user_id: user.id,
                            role: MembershipRole::PrimaryOwner,
                        };
                        let _ = Membership::insert(&m, &conn, &logger).unwrap();
                        return Ok(namespace.uuid.to_string());
                    }
                    Err(Error::RollbackTransaction)
                });
            if let Ok(uuid) = result {
                return res.format(json!({"namespace": {
                    "uuid": uuid,
                }}));
            }
            res.status(Status::InternalServerError)
        },
    }
}
