use fourche::queue::Queue;
use rocket::http::{ContentType, Header, Status};

use eloquentlog_backend_api::job;

use run_test;

#[test]
fn test_login_with_wrong_username() {
    run_test(|client, conn, _, _| {
        let email = "johnny@example.org";
        let password = "pa$$w0rD";

        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                  "email": "{}",
                  "username": "johnny",
                  "password": "{}"
                }}"#,
                &email, &password,
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
            .patch(format!("/_api/user/activate/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                  "username": "{}",
                  "password": "{}"
                }}"#,
                "hennry", &password,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
    });
}

#[test]
fn test_login_with_wrong_password() {
    run_test(|client, conn, _, _| {
        let email = "johnny@example.org";
        let password = "pa$$w0rD";

        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                  "email": "{}",
                  "username": "johnny",
                  "password": "{}"
                }}"#,
                &email, &password,
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
            .patch(format!("/_api/user/activate/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                  "username": "{}",
                  "password": "{}"
                }}"#,
                &email, "wrong-password",
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
    });
}

#[test]
fn test_login() {
    run_test(|client, conn, _, _| {
        let email = "johnny@example.org";
        let password = "pa$$w0rD";

        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                  "email": "{}",
                  "username": "johnny",
                  "password": "{}"
                }}"#,
                &email, &password,
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
            .patch(format!("/_api/user/activate/{}", session_id))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                  "username": "{}",
                  "password": "{}"
                }}"#,
                &email, &password,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
    });
}
