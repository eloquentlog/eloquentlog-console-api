use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use crate::db::DbConn;
use crate::model::message::{AgentType, Message, NewMessage};
use crate::model::user::User;
use crate::response::Response;
use crate::request::message::Message as RequestData;
use crate::validation::message::Validator;

const MESSAGES_PER_REQUEST: i64 = 100;

pub mod preflight {
    use rocket::response::Response as RawResponse;
    use rocket_slog::SyncLogger;

    use crate::response::no_content_for;

    #[options("/message/<namespace_key>/append/<stream_slug>")]
    pub fn append<'a>(
        namespace_key: String,
        stream_slug: String,
        logger: SyncLogger,
    ) -> RawResponse<'a>
    {
        info!(
            logger,
            "namespace: {}, stream: {}", namespace_key, stream_slug
        );
        no_content_for("POST")
    }

    #[options("/message/<namespace_key>/lrange/<stream_slug>/<start>/<stop>")]
    pub fn lrange<'a>(
        namespace_key: String,
        stream_slug: String,
        start: i64,
        stop: i64,
        logger: SyncLogger,
    ) -> RawResponse<'a>
    {
        info!(
            logger,
            "namespace: {}, stream: {}, start: {}, stop: {}",
            namespace_key,
            stream_slug,
            start,
            stop
        );
        no_content_for("GET")
    }
}

// Save a new log message.
//
// ## TODO: Move ingest API
//
// The value looks like this:
//
// ```json
// {
//    "title": "",
//    "level": 0,
//    "content": ""
//    ...
// }
// ```
#[post(
    "/message/<namespace_key>/append/<stream_slug>",
    format = "json",
    data = "<data>"
)]
pub fn append(
    user: &User,
    namespace_key: String,
    stream_slug: String,
    data: Json<RequestData>,
    conn: DbConn,
    logger: SyncLogger,
) -> Response
{
    let res: Response = Default::default();

    info!(
        logger,
        "user: {}, namespace: {}, stream: {}",
        user.uuid,
        namespace_key,
        stream_slug
    );

    // FIXME
    // * namespace
    // * validations for stream_id (slug) and agent_* fields
    let v = Validator::new(&data, &logger);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            // FIXME
            let stream_id = 1;
            let mut m = NewMessage::from(data.0.clone());
            m.stream_id = stream_id;
            m.agent_id = user.id;
            m.agent_type = AgentType::Person;
            if let Some(id) = Message::insert(&m, &conn, &logger) {
                info!(logger, "user: {}", user.uuid);
                return res.format(json!({"message": {
                    "id": id,
                }}));
            }
            res.status(Status::InternalServerError)
        },
    }
}

#[get("/message/<namespace_key>/lrange/<stream_slug>/<start>/<stop>")]
pub fn lrange(
    user: &User,
    namespace_key: String,
    stream_slug: String,
    start: u64,
    stop: u64,
    conn: DbConn,
    logger: SyncLogger,
) -> Response
{
    let res: Response = Default::default();

    info!(
        logger,
        "user: {}, namespace: {}, stream: {}, start: {}, stop: {}",
        user.uuid,
        namespace_key,
        stream_slug,
        start,
        stop
    );

    // TODO
    let offset = start as i64;
    let mut limit = (stop - start + 2) as i64;
    if limit < 1 {
        limit = 1;
    }

    // FIXME
    // * visible to user (and use namespace_key)

    let data = match Message::fetch_by_stream_slug(
        stream_slug,
        offset,
        limit,
        &conn,
        &logger,
    ) {
        None => {
            error!(logger, "err: not found user.id {}", user.uuid);
            vec![]
        },
        Some(a) => a.iter().map(|m| json!({ "message": m })).collect(),
    };
    res.format(json!(data))
}
