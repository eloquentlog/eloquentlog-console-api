use diesel::{self, prelude::*};
use chrono::{Utc, TimeZone};
use rocket::http::{ContentType, Header, Status};
use serde_json::Value;
use uuid::Uuid;

use eloquentlog_console_api::model;

use crate::{
    minify, run_test, load_user, make_raw_password, NAMESPACES, STREAMS, USERS,
};

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

        let namespace_key = "key";
        let stream_slug = "slug";

        let mut res = client
            .get(format!(
                "/v1/message/{}/lrange/{}/0/2",
                namespace_key, stream_slug
            ))
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

        let ns = NAMESPACES.get("piano").unwrap();
        let _namespace_id =
            diesel::insert_into(model::namespace::namespaces::table)
                .values(ns)
                .returning(model::namespace::namespaces::id)
                .get_result::<i64>(conn.db)
                .unwrap_or_else(|_| panic!("Error inserting: {}", ns));

        let s = STREAMS.get("oswald's stream").unwrap().clone();
        // s.namespace_id = namespace_id;
        let stream_id = diesel::insert_into(model::stream::streams::table)
            .values(s)
            .returning(model::stream::streams::id)
            .get_result::<i64>(conn.db)
            .unwrap_or_else(|_| panic!("Error inserting: {}", s));

        // 2019-08-07T06:05:04.333
        let dt = Utc.ymd(2019, 8, 7).and_hms_milli(6, 5, 4, 333);
        let m = model::message::Message {
            id: 1,
            agent_id: user.id,
            agent_type: model::message::AgentType::Person,
            stream_id,
            code: None,
            lang: "en".to_string(),
            level: model::message::LogLevel::Information,
            format: model::message::LogFormat::TOML,
            title: "title".to_string(),
            content: None,
            created_at: dt.naive_utc(),
            updated_at: dt.naive_utc(),
        };

        let id = diesel::insert_into(model::message::messages::table)
            .values(&m)
            .returning(model::message::messages::id)
            .get_result::<i64>(conn.db)
            .unwrap_or_else(|_| panic!("Error inserting: {}", m));

        let namespace_key = "key";
        let stream_slug = "slug"; // FIXME

        let mut res = client
            .get(format!(
                "/v1/message/{}/lrange/{}/0/2",
                namespace_key, stream_slug
            ))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        assert_eq!(
            res.body_string().unwrap(),
            minify(format!(
                r#"[{{
"message": {{
  "agent_id": 1,
  "agent_type": "Person",
  "code": null,
  "content": null,
  "created_at": "2019-08-07T06:05:04.333",
  "format": "TOML",
  "id": {},
  "lang": "en",
  "level": "Information",
  "stream_id": 1,
  "title": "title",
  "updated_at": "2019-08-07T06:05:04.333"
}}
}}]"#,
                id,
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

        let namespace_key = "key";
        let stream_slug = "slug";

        let mut res = client
            .post(format!(
                "/v1/message/{}/append/{}",
                namespace_key, stream_slug
            ))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(
                r#"{
                    "agent_id": 1,
                    "agent_type": "person",
                    "stream_id": 1,
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

        let namespace_key = "key";
        let stream_slug = "slug";

        let ns = NAMESPACES.get("piano").unwrap().clone();
        let _namespace_id =
            diesel::insert_into(model::namespace::namespaces::table)
                .values(ns)
                .returning(model::namespace::namespaces::id)
                .get_result::<i64>(conn.db)
                .unwrap_or_else(|_| panic!("Error inserting: {}", ns));

        let s = STREAMS.get("oswald's stream").unwrap().clone();
        // s.namespace_id = namespace_id;
        let stream_uuid = diesel::insert_into(model::stream::streams::table)
            .values(s)
            .returning(model::stream::streams::uuid)
            .get_result::<Uuid>(conn.db)
            .unwrap_or_else(|_| panic!("Error inserting: {}", s));

        let mut res = client
            .post(format!(
                "/v1/message/{}/append/{}",
                namespace_key, stream_slug
            ))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                    "agent_id": 1,
                    "agent_type": "person",
                    "stream_id": 1,
                    "code": "200",
                    "format": "toml",
                    "stream": "{}",
                    "title": "New message",
                    "content": "Hello, world!"
                }}"#,
                stream_uuid
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("id"));
    });
}
