use rocket::http::{ContentType, Status};

use eloquentlog_backend_api::model::user;

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
          "password": "password"
        }}"#,
                &email,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::UnprocessableEntity);

        // TODO
        assert!(user::User::find_by_email(&email, &db_conn, &logger).is_none());
    });
}

#[test]
fn test_register() {
    run_test(|client, db_conn, _, _, logger| {
        let email = "postmaster@example.org";
        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
          "email": "{}",
          "password": "pa$$w0rD"
        }}"#,
                &email,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        // TODO
        let u = user::User::find_by_email(&email, &db_conn, &logger).unwrap();
        assert_eq!(u.state, user::UserState::Pending);
        assert_eq!(u.email, email);
    });
}
