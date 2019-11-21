use fourche::queue::Queue;
use rocket::http::{ContentType, Header, Status};
use redis::{Commands, RedisError};

use eloquentlog_backend_api::model;
use eloquentlog_backend_api::job;

use crate::run_test;

#[test]
fn test_register_with_validation_error() {
    run_test(|client, conn, _, logger| {
        let email = "postmaster@example.org";
        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                  "email": "{}",
                  "username": "hennry",
                  "password": "password"
                }}"#,
                &email,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::UnprocessableEntity);
        assert!(model::user::User::find_by_email(&email, conn.db, &logger)
            .is_none());
    });
}

#[test]
fn test_register() {
    run_test(|client, conn, _, logger| {
        let email = "postmaster@example.org";
        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                  "email": "{}",
                  "username": "hennry",
                  "password": "pa$$w0rD"
                }}"#,
                &email,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let result = model::user::User::find_by_email(email, conn.db, logger);
        assert!(result.is_none());

        // TODO: check sent email
        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendUserActivationEmail);
        assert!(!job.args.is_empty());

        let session_id = job.args[1].to_string();
        let key = format!("ur-{}", session_id);

        let result: Result<String, RedisError> = conn.ss.get(key);
        assert!(result.is_ok());
    });
}
