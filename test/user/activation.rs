use fourche::queue::Queue;
use rocket::http::{ContentType, Header, Status};

use eloquentlog_backend_api::model;
use eloquentlog_backend_api::job;

use super::super::run_test;

#[test]
fn test_activate_with_invalid_token() {
    run_test(|client, conn, _, logger| {
        let email = "hennry@example.org";
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

        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendUserActivationEmail);
        assert!(!job.args.is_empty());

        let token = "invalid-token";
        let session_id = job.args[2].to_string();

        let res = client
            .post(format!("/_api/user/activate?s={}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::BadRequest);

        let user =
            model::user::User::find_by_email(email, conn.db, logger).unwrap();
        assert_eq!(user.state, model::user::UserState::Pending);
    });
}

#[test]
fn test_activate_with_invalid_session_id() {
    run_test(|client, conn, _, logger| {
        let email = "hennry@example.org";
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

        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendUserActivationEmail);
        assert!(!job.args.is_empty());

        let token = job.args[1].to_string();
        let session_id = "invalid-session_id";

        let res = client
            .post(format!("/_api/user/activate?s={}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::BadRequest);

        let user =
            model::user::User::find_by_email(email, conn.db, logger).unwrap();
        assert_eq!(user.state, model::user::UserState::Pending);
    });
}

#[test]
fn test_activate() {
    run_test(|client, conn, _, logger| {
        let email = "hennry@example.org";
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

        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendUserActivationEmail);
        assert!(!job.args.is_empty());

        let token = job.args[1].to_string();
        let session_id = job.args[2].to_string();

        let res = client
            .post(format!("/_api/user/activate?s={}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let user =
            model::user::User::find_by_email(email, conn.db, logger).unwrap();
        assert_eq!(user.state, model::user::UserState::Active);
    });
}
