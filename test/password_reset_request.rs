use fourche::queue::Queue;
use rocket::http::{ContentType, Header, Status};
use redis::{Commands, RedisError};

use eloquentlog_console_api::model;
use eloquentlog_console_api::job;

use crate::{run_test, load_user, USERS};

#[test]
fn test_password_reset_request_with_validation_error() {
    run_test(|client, conn, _, logger| {
        let u = USERS.get("oswald").unwrap().clone();
        let user = load_user(u, conn.db);

        let email = "invalid";

        let _ = client
            .head("/_api/password/reset/")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let res = client
            .put("/_api/passord/reset")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                  "email": "{}"
                }}"#,
                &email,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::NotFound);
        let result =
            model::user::User::find_by_email(&user.email, conn.db, logger);
        assert!(result.unwrap().reset_password_token.is_none());
    });
}

#[test]
fn test_password_reset_request() {
    run_test(|client, conn, _, logger| {
        let u = USERS.get("oswald").unwrap().clone();
        let user = load_user(u, conn.db);

        let email = user.email;

        let _ = client
            .head("/_api/password/reset/")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let res = client
            .put("/_api/password/reset")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                  "email": "{}"
                }}"#,
                &email,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let result = model::user::User::find_by_email(&email, conn.db, logger);
        assert!(result.unwrap().reset_password_token.is_some());

        // TODO: check sent email
        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendPasswordResetEmail);
        assert!(!job.args.is_empty());

        let session_id = job.args[1].to_string();
        let key = format!("pr-{}", session_id);
        let value: Result<String, RedisError> = conn.ss.get(key);
        assert!(value.is_ok());
    });
}
