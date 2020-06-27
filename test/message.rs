use diesel::{self, prelude::*};
use chrono::{Utc, TimeZone};
use rocket::http::{ContentType, Header, Status};
use serde_json::Value;

use eloquentlog_console_api::model;

use crate::{minify, run_test, load_user, make_raw_password, USERS};

#[test]
fn test_lrange_no_message() {
    run_test(|client, conn, _, _| {
        let u = USERS.get("oswald").unwrap().clone();
        let password = make_raw_password(&u);
        let user = load_user(u, conn.db);

        let _ = client
            .head("/_api/login/")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let mut res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                    "username": "{}",
                    "password": "{}"
                }}"#,
                user.email, password,
            ))
            .dispatch();

        let body = res.body_string().unwrap();
        let result: Value = serde_json::from_str(&body).unwrap();
        let token = result["token"].as_str().unwrap();

        let mut res = client
            .get("/_api/message/lrange/namespace/0/2")
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("[]"));
    });
}

#[test]
fn test_lrange_messages() {
    run_test(|client, conn, _, _| {
        let u = USERS.get("oswald").unwrap().clone();
        let password = make_raw_password(&u);
        let user = load_user(u, conn.db);

        let _ = client
            .head("/_api/login/")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let mut res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                    "username": "{}",
                    "password": "{}"
                }}"#,
                user.email, password,
            ))
            .dispatch();

        let body = res.body_string().unwrap();
        let result: Value = serde_json::from_str(&body).unwrap();
        let token = result["token"].as_str().unwrap();

        // 2019-08-07T06:05:04.333
        let dt = Utc.ymd(2019, 8, 7).and_hms_milli(6, 5, 4, 333);
        let m = model::message::Message {
            id: 1,
            code: None,
            lang: "en".to_string(),
            level: model::message::LogLevel::Information,
            format: model::message::LogFormat::TOML,
            title: "title".to_string(),
            content: None,
            created_at: dt.naive_utc(),
            updated_at: dt.naive_utc(),
            user_id: user.id,
        };

        let id = diesel::insert_into(model::message::messages::table)
            .values(&m)
            .returning(model::message::messages::id)
            .get_result::<i64>(conn.db)
            .unwrap_or_else(|_| panic!("Error inserting: {}", m));

        let mut res = client
            .get("/_api/message/lrange/namespace/0/2")
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        assert_eq!(
            res.body_string().unwrap(),
            minify(format!(
                r#"[{{
"message": {{
  "code": null,
  "content": null,
  "created_at": "2019-08-07T06:05:04.333",
  "format": "TOML",
  "id": {},
  "lang": "en",
  "level": "Information",
  "title": "title",
  "updated_at": "2019-08-07T06:05:04.333",
  "user_id": {}
}}
}}]"#,
                id, user.id
            ))
        );
    });
}

#[test]
fn test_append_with_validation_errors() {
    run_test(|client, conn, _, _| {
        let u = USERS.get("oswald").unwrap().clone();
        let password = make_raw_password(&u);
        let user = load_user(u, conn.db);

        let _ = client
            .head("/_api/login/")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let mut res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                    "username": "{}",
                    "password": "{}"
                }}"#,
                user.email, password,
            ))
            .dispatch();

        let body = res.body_string().unwrap();
        let result: Value = serde_json::from_str(&body).unwrap();
        let token = result["token"].as_str().unwrap();

        let mut res = client
            .post("/_api/message/append/namespace")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
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
fn test_append() {
    run_test(|client, conn, _, _| {
        let u = USERS.get("oswald").unwrap().clone();
        let password = make_raw_password(&u);
        let user = load_user(u, conn.db);

        let _ = client
            .head("/_api/login/")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let mut res = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                    "username": "{}",
                    "password": "{}"
                }}"#,
                user.email, password,
            ))
            .dispatch();

        let body = res.body_string().unwrap();
        let result: Value = serde_json::from_str(&body).unwrap();
        let token = result["token"].as_str().unwrap();

        let mut res = client
            .post("/_api/message/append/namespace")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
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
