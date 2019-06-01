use chrono::{Utc, TimeZone};
use diesel::{self, prelude::*};
use diesel::pg::PgConnection;
use rocket::http::{ContentType, Header, Status};

use eloquentlog_backend_api::config;
use eloquentlog_backend_api::model::{message, user};
use eloquentlog_backend_api::request::voucher::AUTHORIZATION_HEADER_KEY;
use eloquentlog_backend_api::logger::Logger;

use {minify, run_test};

fn build_test_user(conn: &PgConnection, logger: &Logger) -> user::User {
    let password = "pa$$w0rD";
    let mut u = user::NewUser {
        name: None,
        username: None,
        email: "foo@example.org".to_string(),

        ..Default::default()
    };
    u.set_password(&password);

    user::User::insert(&u, &conn, &logger)
        .unwrap_or_else(|| panic!("Error inserting: {}", u))
}

fn build_authorization_header<'a>(
    user: &user::User,
    config: &config::Config,
) -> Header<'a>
{
    Header::new(
        AUTHORIZATION_HEADER_KEY,
        user.generate_authorization_voucher(
            &config.authorization_voucher_issuer,
            &config.authorization_voucher_key_id,
            &config.authorization_voucher_secret,
        )
        .to_string(),
    )
}

#[test]
fn test_get_no_message() {
    run_test(|client, conn, config, logger| {
        let user = build_test_user(&conn, &logger);
        let auth = build_authorization_header(&user, &config);

        let mut res = client.get("/_api/messages").header(auth).dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("[]"));
    });
}

#[test]
fn test_get_recent_messages() {
    run_test(|client, conn, config, logger| {
        let user = build_test_user(&conn, &logger);
        let auth = build_authorization_header(&user, &config);

        let dt = Utc.ymd(2019, 8, 7).and_hms_milli(6, 5, 4, 333); // 2019-08-07T06:05:04.333
        let m = message::Message {
            id: 1,
            code: None,
            lang: "en".to_string(),
            level: message::LogLevel::Information,
            format: message::LogFormat::TOML,
            title: "title".to_string(),
            content: None,
            created_at: dt.naive_utc(),
            updated_at: dt.naive_utc(),
        };

        let id = diesel::insert_into(message::messages::table)
            .values(&m)
            .returning(message::messages::id)
            .get_result::<i64>(conn)
            .unwrap_or_else(|_| panic!("Error inserting: {}", m));

        let mut res = client.get("/_api/messages").header(auth).dispatch();

        assert_eq!(res.status(), Status::Ok);

        assert_eq!(
            res.body_string().unwrap(),
            minify(format!(
                r#"{{
"messages": [{{
  "code": null,
  "content": null,
  "created_at": "2019-08-07T06:05:04.333",
  "format": "TOML",
  "id": {},
  "lang": "en",
  "level": "Information",
  "title": "title",
  "updated_at": "2019-08-07T06:05:04.333"
}}]
}}"#,
                id
            ))
        );
    });
}

#[test]
fn test_post_with_validation_errors() {
    run_test(|client, conn, config, logger| {
        let user = build_test_user(&conn, &logger);
        let auth = build_authorization_header(&user, &config);

        let mut res = client
            .post("/_api/messages")
            .header(ContentType::JSON)
            .header(auth)
            .body(
                r#"{
        "code": "",
        "title": "New Message",
        "content": "Hello, world!"
      }"#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::UnprocessableEntity);
        assert!(res.body_string().unwrap().contains("errors"));
    });
}

#[test]
fn test_post() {
    run_test(|client, conn, config, logger| {
        let user = build_test_user(&conn, &logger);
        let auth = build_authorization_header(&user, &config);

        let mut res = client
            .post("/_api/messages")
            .header(ContentType::JSON)
            .header(auth)
            .body(
                r#"{
        "format": "toml",
        "code": "200",
        "title": "New message",
        "content": "Hello, world!"
      }"#,
            )
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("id"));
    });
}

#[test]
fn test_put() {
    run_test(|client, conn, config, logger| {
        let user = build_test_user(&conn, &logger);
        let auth = build_authorization_header(&user, &config);

        let m = message::NewMessage {
            code: None,
            lang: "en".to_string(),
            level: message::LogLevel::Information,
            format: message::LogFormat::TOML,
            title: Some("title".to_string()),
            content: None,
        };

        let id = diesel::insert_into(message::messages::table)
            .values(&m)
            .returning(message::messages::id)
            .get_result::<i64>(conn)
            .unwrap_or_else(|_| panic!("Error inserting: {}", m));

        let mut res = client
            .put(format!("/_api/messages/{}", id))
            .header(ContentType::JSON)
            .header(auth)
            .body(format!(
                r#"{{
        "id": {},
        "title": "Updated message",
        "content": "Hello, world!"
      }}"#,
                id,
            ))
            .dispatch();

        let result = message::messages::table
            .find(id)
            .first::<message::Message>(conn)
            .unwrap();
        assert_eq!("Updated message", result.title);
        assert_eq!("Hello, world!", result.content.unwrap());

        assert_eq!(res.status(), Status::Ok);
        assert!(res
            .body_string()
            .unwrap()
            .contains(&format!("\"id\":{}", id)));
    });
}
