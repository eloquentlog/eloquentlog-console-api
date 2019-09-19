use rocket::http::{ContentType, Status};

use eloquentlog_backend_api::model;

use run_test;

#[test]
fn test_login_with_username() {
    run_test(|client, conn, _, logger| {
        let password = "pa$$w0rD";
        let mut u = model::user::NewUser {
            name: None,
            username: "johnny".to_string(),
            email: "johnny@example.org".to_string(),

            ..Default::default()
        };
        u.set_password(&password);

        model::user::User::insert(&u, &conn.db, &logger)
            .unwrap_or_else(|| panic!("Error inserting: {}", u));

        // Not available
        let res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                  "username": "{}",
                  "password": "{}"
                }}"#,
                u.username, password,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
    });
}

#[test]
fn test_login_with_wrong_password() {
    run_test(|client, conn, _, logger| {
        let password = "pa$$w0rD";
        let mut u = model::user::NewUser {
            name: None,
            username: "johnny".to_string(),
            email: "johnny@example.org".to_string(),

            ..Default::default()
        };
        u.set_password(&password);

        model::user::User::insert(&u, &conn.db, &logger)
            .unwrap_or_else(|| panic!("Error inserting: {}", u));

        let res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                  "username": "{}",
                  "password": "{}"
                }}"#,
                u.email, "wrong-password",
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
    });
}

#[test]
fn test_login() {
    run_test(|client, conn, _, logger| {
        let password = "pa$$w0rD";
        let mut u = model::user::NewUser {
            name: None,
            username: "johnny".to_string(),
            email: "johnny@example.org".to_string(),

            ..Default::default()
        };
        u.set_password(&password);

        model::user::User::insert(&u, &conn.db, &logger)
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
