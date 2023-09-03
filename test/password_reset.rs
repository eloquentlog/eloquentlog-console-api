use fourche::queue::Queue;
use rocket::http::{ContentType, Header, Status};
use rocket::local::Client;

use eloquentlog_console_api::model;
use eloquentlog_console_api::job;

use crate::{run_test, load_user, USERS};

fn password_reset_request_by(
    user: &model::user::User,
    client: &Client,
) -> Result<(), ()> {
    let _ = client
        .head("/_/password/reset")
        .header(ContentType::JSON)
        .header(Header::new("X-Requested-With", "XMLHttpRequest"))
        .body("{}")
        .dispatch();

    let res = client
        .put("/_/password/reset")
        .header(ContentType::JSON)
        .header(Header::new("X-Requested-With", "XMLHttpRequest"))
        .body(format!(
            r#"{{
              "email": "{}"
            }}"#,
            &user.email,
        ))
        .dispatch();
    match res.status() {
        Status::Ok => Ok(()),
        _ => Err(()),
    }
}

#[test]
fn test_password_reset_with_invalid_token() {
    run_test(|client, conn, _, logger| {
        let u = USERS.get("oswald").unwrap().clone();
        let user = load_user(u, conn.db);

        let request = password_reset_request_by(&user, client);
        assert!(request.is_ok());

        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendPasswordResetEmail);
        assert!(!job.args.is_empty());

        let session_id = job.args[1].to_string();
        let token = "invalid-token";

        let res = client
            .get(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .dispatch();
        assert_eq!(res.status(), Status::NotFound);

        let _ = client
            .head(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let res = client
            .patch(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(
                r#"{
                  "new_password": "pa$$w0rD2"
                }"#,
            )
            .dispatch();
        assert_eq!(res.status(), Status::NotFound);

        let result =
            model::user::User::find_by_email(&user.email, conn.db, logger);
        assert!(result.unwrap().reset_password_token.is_some());
    });
}

#[test]
fn test_password_reset_with_invalid_session_id() {
    run_test(|client, conn, _, logger| {
        let u = USERS.get("oswald").unwrap().clone();
        let user = load_user(u, conn.db);

        let request = password_reset_request_by(&user, client);
        assert!(request.is_ok());

        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendPasswordResetEmail);
        assert!(!job.args.is_empty());

        let session_id = "invalid-session_id";
        let token = job.args[2].to_string();

        let res = client
            .get(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .dispatch();
        assert_eq!(res.status(), Status::NotFound);

        let _ = client
            .head(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let res = client
            .patch(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(
                r#"{
                  "new_password": "pa$$w0rD2"
                }"#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::NotFound);

        let result =
            model::user::User::find_by_email(&user.email, conn.db, logger);
        assert!(result.unwrap().reset_password_token.is_some());
    });
}

#[test]
fn test_password_reset_without_authorization_header() {
    run_test(|client, conn, _, logger| {
        let u = USERS.get("oswald").unwrap().clone();
        let user = load_user(u, conn.db);

        let request = password_reset_request_by(&user, client);
        assert!(request.is_ok());

        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendPasswordResetEmail);
        assert!(!job.args.is_empty());

        let session_id = job.args[1].to_string();

        let res = client
            .get(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .dispatch();
        assert_eq!(res.status(), Status::BadRequest);

        let _ = client
            .head(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let res = client
            .patch(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(
                r#"{
                  "new_password": "pa$$w0rD2"
                }"#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::BadRequest);

        let result =
            model::user::User::find_by_email(&user.email, conn.db, logger);
        assert!(result.unwrap().reset_password_token.is_some());
    });
}

#[test]
fn test_password_reset_without_x_requested_with_header() {
    run_test(|client, conn, _, logger| {
        let u = USERS.get("oswald").unwrap().clone();
        let user = load_user(u, conn.db);

        let request = password_reset_request_by(&user, client);
        assert!(request.is_ok());

        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendPasswordResetEmail);
        assert!(!job.args.is_empty());

        let session_id = job.args[1].to_string();
        let token = job.args[1].to_string();

        let res = client
            .get(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();
        assert_eq!(res.status(), Status::BadRequest);

        let _ = client
            .head(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .body("{}")
            .dispatch();

        let res = client
            .patch(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .body(
                r#"{
                  "new_password": "pa$$w0rD2"
                }"#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::BadRequest);

        let result =
            model::user::User::find_by_email(&user.email, conn.db, logger);
        assert!(result.unwrap().reset_password_token.is_some());
    });
}

#[test]
fn test_password_reset() {
    run_test(|client, conn, _, logger| {
        let u = USERS.get("oswald").unwrap().clone();
        let user = load_user(u, conn.db);

        let request = password_reset_request_by(&user, client);
        assert!(request.is_ok());

        let mut queue = Queue::new("default", conn.mq);
        let job = queue.dequeue::<job::Job<String>>().ok().unwrap();
        assert_eq!(job.kind, job::JobKind::SendPasswordResetEmail);
        assert!(!job.args.is_empty());

        let session_id = job.args[1].to_string();
        let token = job.args[2].to_string();

        let res = client
            .get(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);

        let _ = client
            .head(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let res = client
            .patch(format!("/_/password/reset/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(
                r#"{
                  "new_password": "pa$$w0rD2"
                }"#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let result =
            model::user::User::find_by_email(&user.email, conn.db, logger);
        assert!(result.unwrap().reset_password_token.is_some());
    });
}
