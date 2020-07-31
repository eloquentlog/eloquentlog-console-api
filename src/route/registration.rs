use chrono::{Duration, Utc};
use diesel::result::Error;
use fourche::queue::Queue;
use redis::{Commands, RedisError};
use rocket::State;
use rocket::http::{Cookies, Status};
use rocket_contrib::json::Json;
use rocket_slog::SyncLogger;

use crate::config::Config;
use crate::db::DbConn;
use crate::job::{Job, JobKind};
use crate::model::token::{VerificationClaims, Claims, TokenData};
use crate::model::namespace::{Namespace, NewNamespace};
use crate::model::membership::{Membership, NewMembership};
use crate::model::stream::{Stream, NewStream};
use crate::model::user::{NewUser, User};
use crate::model::user_email::{NewUserEmail, UserEmail};
use crate::mq::MqConn;
use crate::response::Response;
use crate::request::user::registration::UserRegistration;
use crate::validation::user::Validator;
use crate::ss::SsConn;
use crate::util::split_token;

pub mod preflight {
    use rocket::State;
    use rocket::response::Response as RawResponse;

    use crate::config::Config;
    use crate::response::no_content_for;

    #[options("/register")]
    pub fn register<'a>(config: State<Config>) -> RawResponse<'a> {
        no_content_for("HEAD,POST", &config)
    }

    #[options("/deregister")]
    pub fn deregister<'a>(config: State<Config>) -> RawResponse<'a> {
        no_content_for("POST", &config)
    }
}

pub mod preignition {
    use chrono::{Duration, Utc};
    use redis::{Commands, RedisError};
    use rocket::State;
    use rocket::http::{Cookie, Cookies, SameSite, Status};
    use rocket_slog::SyncLogger;

    use crate::config::Config;
    use crate::response::Response;
    use crate::ss::SsConn;
    use crate::util::generate_random_hash;

    #[head("/register", format = "json")]
    pub fn register<'a>(
        config: State<Config>,
        mut cookies: Cookies,
        logger: SyncLogger,
        mut ss_conn: SsConn,
    ) -> Response<'a>
    {
        // returns CSRF token
        let res: Response = Default::default();
        info!(logger, "preignition");

        let duration = Duration::minutes(Config::CSRF_HASH_DURATION);
        let expires_at = (Utc::now() + duration).timestamp();
        let key_value = generate_random_hash(
            Config::CSRF_HASH_SOURCE,
            Config::CSRF_HASH_LENGTH,
        );
        let key = format!("xs-{}", key_value);
        let value = "1";
        let result: Result<String, RedisError> = ss_conn
            .set_ex(&key, value, expires_at as usize)
            .map_err(|e| {
                error!(logger, "error: {}", e);
                e
            });
        if result.is_ok() {
            let mut cookie = Cookie::new("csrf_token", key);
            cookie.set_http_only(true);
            cookie.set_secure(config.cookie_secure);
            cookie.set_same_site(SameSite::Strict);
            // encrypted value with expires 1 week from now
            cookies.add_private(cookie);
            return res.status(Status::Ok);
        }
        error!(logger, "something went wrong on register");
        res.status(Status::InternalServerError)
    }
}

#[post("/register", data = "<data>", format = "json")]
pub fn register<'a>(
    data: Json<UserRegistration>,
    mut cookies: Cookies,
    db_conn: DbConn,
    mut mq_conn: MqConn,
    mut ss_conn: SsConn,
    logger: SyncLogger,
    config: State<Config>,
) -> Response<'a>
{
    // FIXME: create `account_registrar` service
    let res: Response = Default::default();

    let cookie = cookies.get_private("csrf_token").ok_or("");
    if cookie.is_err() {
        info!(logger, "error: missing csrf_token");
        return res.status(Status::Unauthorized).format(json!({
            "message": "The CSRF token is required."
        }));
    }
    let key = cookie.ok().unwrap().value().to_string();
    let result: Result<i64, RedisError> = ss_conn.get(&key).map_err(|e| {
        error!(logger, "error: {}", e);
        e
    });
    if result.is_err() {
        return res.status(Status::Unauthorized).format(json!({
            "message": "The CSRF token has been expired. Reload the page."
        }));
    }

    let v = Validator::new(&db_conn, &data, &logger);
    match v.validate() {
        Err(errors) => {
            res.status(Status::UnprocessableEntity).format(json!({
                "errors": errors,
            }))
        },
        Ok(_) => {
            // TODO:
            // impl service object handles token generation/activation.
            // see also login
            let now = Utc::now();
            let granted_at = now.timestamp();
            let expires_at = (now + Duration::hours(1)).timestamp();

            let result: Result<(i64, String), Error> = db_conn
                .build_transaction()
                .serializable()
                .deferrable()
                .read_write()
                .run::<(i64, String), diesel::result::Error, _>(|| {
                    let mut u = NewUser::from(&data.0);
                    u.set_password(&data.password);
                    let user = User::insert(&u, &db_conn, &logger).unwrap();
                    let ue = NewUserEmail::from(&user);
                    let user_email =
                        UserEmail::insert(&ue, &db_conn, &logger).unwrap();

                    {
                        // TODO: async
                        let mut ns = NewNamespace::default();
                        ns.name = format!("{}'s default namespace", u.username);
                        let namespace =
                            Namespace::insert(&ns, &db_conn, &logger).unwrap();

                        let mut s = NewStream::default();
                        s.namespace_id = namespace.id;
                        let _ = Stream::insert(&s, &db_conn, &logger).unwrap();

                        let mut m = NewMembership::default();
                        m.namespace_id = namespace.id;
                        m.user_id = user.id;
                        let _ =
                            Membership::insert(&m, &db_conn, &logger).unwrap();
                    }

                    let data = TokenData {
                        value: UserEmail::generate_token(),
                        granted_at,
                        expires_at,
                    };
                    let raw_token = VerificationClaims::encode(
                        data,
                        &config.verification_token_issuer,
                        &config.verification_token_key_id,
                        &config.verification_token_secret,
                    );

                    if let Err(e) = user_email
                        .grant_token::<VerificationClaims>(
                            &raw_token,
                            &config.verification_token_issuer,
                            &config.verification_token_secret,
                            &db_conn,
                            &logger,
                        )
                    {
                        error!(logger, "error: {}", e);
                        return Err(Error::RollbackTransaction);
                    }
                    Ok((user_email.id, raw_token))
                });

            if let Ok((id, raw_token)) = result {
                if let Some((token, sign)) = split_token(raw_token) {
                    // TODO: use general value
                    let session_id = UserEmail::generate_token();
                    let key = format!("ua-{}", session_id);

                    // Instead of saving the signature into a cookie,
                    // putting it in session store.
                    //
                    // Because we need to make it available users to activate
                    // the account also via another device than signed up, so
                    // we can't rely on a cookie of http client (browser).
                    let result: Result<String, RedisError> = ss_conn
                        .set_ex(&key, sign, expires_at as usize)
                        .map_err(|e| {
                            error!(logger, "error: {}", e);
                            e
                        });

                    if result.is_ok() {
                        let job = Job::<String> {
                            kind: JobKind::SendUserActivationEmail,
                            args: vec![id.to_string(), session_id, token],
                        };
                        let mut queue = Queue::new("default", &mut mq_conn);
                        if let Err(err) = queue.enqueue::<Job<String>>(job) {
                            error!(logger, "error: {}", err);
                        } else {
                            return res;
                        }
                    }
                }
            }
            res.status(Status::InternalServerError).format(json!({
                "message": "Something wrong happen, sorry :'("
            }))
        },
    }
}

#[post("/deregister", format = "json")]
pub fn deregister<'a>(
    mut cookies: Cookies,
    user: &User,
    mut ss_conn: SsConn,
    logger: SyncLogger,
) -> Response<'a>
{
    let res: Response = Default::default();

    info!(logger, "user: {}", user.uuid);

    let cookie = cookies.get_private("csrf_token").ok_or("");
    if cookie.is_err() {
        info!(logger, "error: missing csrf_token");
        return res.status(Status::Unauthorized).format(json!({
            "message": "The CSRF token is required."
        }));
    }
    let key = cookie.ok().unwrap().value().to_string();
    let result: Result<i64, RedisError> = ss_conn.get(&key).map_err(|e| {
        error!(logger, "error: {}", e);
        e
    });
    if result.is_err() {
        return res.status(Status::Unauthorized).format(json!({
            "message": "The CSRF token has been expired. Reload the page."
        }));
    }

    // TODO
    res.status(Status::UnprocessableEntity)
}
