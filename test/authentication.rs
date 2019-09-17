use rocket::http::{ContentType, Status};

use eloquentlog_backend_api::model::user;

use run_test;

#[test]
fn test_login() {
    run_test(|client, conn, _, logger| {
        let password = "pa$$w0rD";
        let mut u = user::NewUser {
            name: None,
            username: "johnny".to_string(),
            email: "johnny@example.org".to_string(),

            ..Default::default()
        };
        u.set_password(&password);

        user::User::insert(&u, &conn.db, &logger)
            .unwrap_or_else(|| panic!("Error inserting: {}", u));

        let res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                  "username": "{}",
                  "password": "{}"
                }}"#,
                u.email, password,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
    });
}
