use rocket::http::{ContentType, Status};

use run_test;

#[test]
fn test_register_with_validation_error() {
    run_test(|client, _, _| {
        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
            .body(
                r#"{
          "email": "postmaster@example.org",
          "password": "password"
        }"#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::UnprocessableEntity);
    });
}

#[test]
fn test_register() {
    run_test(|client, _, _| {
        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
            .body(
                r#"{
          "email": "postmaster@example.org",
          "password": "pa$$w0rD"
        }"#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
    });
}
