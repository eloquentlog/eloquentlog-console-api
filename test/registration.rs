use fourche::queue::Queue;
use rocket::http::{ContentType, Status};

use eloquentlog_backend_api::model::user;
use eloquentlog_backend_api::job::{Job, JobKind};

use run_test;

#[test]
fn test_register_with_validation_error() {
    run_test(|client, db_conn, _, _, logger| {
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
        assert!(user::User::find_by_email(&email, &db_conn, &logger).is_none());
    });
}

#[test]
fn test_register() {
    run_test(|client, db_conn, mut mq_conn, _, logger| {
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

        let u = user::User::find_by_email(&email, &db_conn, &logger).unwrap();
        assert_eq!(u.state, user::UserState::Pending);
        assert_eq!(u.email, email);

        // TODO: check sent email
        let mut queue = Queue::new("default", &mut mq_conn);
        let job = queue.dequeue::<Job<String>>().ok().unwrap();
        assert_eq!(job.kind, JobKind::SendUserActivationEmail);
        assert!(!job.args.is_empty());
    });
}
