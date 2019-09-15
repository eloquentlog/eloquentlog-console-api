use fourche::queue::Queue;
use rocket::http::{ContentType, Status};
use redis::{Commands, RedisError};

use eloquentlog_backend_api::model::user;
use eloquentlog_backend_api::job::{Job, JobKind};

use run_test;

#[test]
fn test_register_with_validation_error() {
    run_test(|client, conn_ref, _, logger| {
        let email = "postmaster@example.org";
        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
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
        assert!(
            user::User::find_by_email(&email, conn_ref.db, &logger).is_none()
        );
    });
}

#[test]
fn test_register() {
    run_test(|client, conn_ref, _, logger| {
        let email = "postmaster@example.org";
        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
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

        let u =
            user::User::find_by_email(&email, conn_ref.db, &logger).unwrap();
        assert_eq!(u.state, user::UserState::Pending);
        assert_eq!(u.email, email);

        // TODO: check sent email
        let mut queue = Queue::new("default", conn_ref.mq);
        let job = queue.dequeue::<Job<String>>().ok().unwrap();
        assert_eq!(job.kind, JobKind::SendUserActivationEmail);
        assert!(!job.args.is_empty());

        let sessid = job.args[2].to_string();
        let result: Result<String, RedisError> = conn_ref.ss.get(sessid);
        assert!(result.is_ok());
    });
}
