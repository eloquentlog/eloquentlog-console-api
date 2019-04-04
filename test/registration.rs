use rocket::http::{ContentType, Status};

use run_test;

#[test]
fn test_register() {
    run_test(|client, _| {
        let res = client
            .post("/_api/register")
            .header(ContentType::JSON)
            .body(
                r#"{
          "email": "postmaster@example.org",
          "password": "pa$$w0rd"
        }"#,
            )
            .dispatch();

        // TODO
        assert_eq!(res.status(), Status::InternalServerError);
    });
}
