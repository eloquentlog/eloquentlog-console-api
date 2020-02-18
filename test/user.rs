use fourche::queue::Queue;
use rocket::http::{ContentType, Header, Status};

use eloquentlog_console_api::model;
use eloquentlog_console_api::job;

use super::run_test;

#[test]
fn test_user_activate_with_invalid_token() {
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

        let session_id = job.args[1].to_string();
        let token = "invalid-token";

        let res = client
            .patch(format!("/_api/user/activate/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::BadRequest);

        let result = model::user::User::find_by_email(email, conn.db, logger);
        assert!(result.is_none());
    });
}

#[test]
fn test_user_activate_with_invalid_session_id() {
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

        let session_id = "invalid-session_id";
        let token = job.args[2].to_string();

        let res = client
            .patch(format!("/_api/user/activate/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::NotFound);

        let result = model::user::User::find_by_email(email, conn.db, logger);
        assert!(result.is_none());
    });
}

#[test]
fn test_user_activate_without_authorization_header() {
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

        let session_id = job.args[1].to_string();

        let res = client
            .patch(format!("/_api/user/activate/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::BadRequest);

        let result = model::user::User::find_by_email(email, conn.db, logger);
        assert!(result.is_none());
    });
}

#[test]
fn test_user_activate_without_x_requested_with_header() {
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

        let session_id = job.args[1].to_string();
        let token = job.args[2].to_string();

        let res = client
            .patch(format!("/_api/user/activate/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::BadRequest);

        let result = model::user::User::find_by_email(email, conn.db, logger);
        assert!(result.is_none());
    });
}

#[test]
fn test_user_activate() {
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

        let session_id = job.args[1].to_string();
        let token = job.args[2].to_string();

        let res = client
            .patch(format!("/_api/user/activate/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let result = model::user::User::find_by_email(email, conn.db, logger);
        assert!(result.is_some());

        let user = result.unwrap();
        assert_eq!(user.state, model::user::UserState::Active);

        let result =
            model::access_token::AccessToken::find_disabled_personal_token_by_user_id(
                user.id, conn.db, logger,
            );
        assert!(result.is_some());
    });
}
