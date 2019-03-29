use regex::Regex;

use chrono::{Utc, TimeZone};
use diesel::{self, prelude::*};
use rocket::http::{ContentType, Status};

use eloquentlog_backend_api::model::message;

use run_test;

/// Formats JSON text as one line
fn minify(s: String) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\n\s{2}|\n|(:)\s").unwrap();
    }
    RE.replace_all(&s, "$1").to_string()
}

#[test]
fn test_get_no_message() {
    run_test(|client, _| {
        let mut res = client.get("/api/messages").dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("[]"));
    })
}

#[test]
fn test_get_recent_messages() {
    run_test(|client, conn| {
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

        let mut res = client.get("/api/messages").dispatch();

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
    })
}

#[test]
fn test_post_with_validation_errors() {
    run_test(|client, _| {
        let mut res = client
            .post("/api/messages")
            .header(ContentType::JSON)
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
    })
}

#[test]
fn test_post() {
    run_test(|client, _| {
        let mut res = client
            .post("/api/messages")
            .header(ContentType::JSON)
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
    })
}

#[test]
fn test_put() {
    run_test(|client, conn| {
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
            .put(format!("/api/messages/{}", id))
            .header(ContentType::JSON)
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
    })
}
