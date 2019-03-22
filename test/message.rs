use diesel::{self, prelude::*};
use diesel::PgConnection;
use rocket::http::{ContentType, Status};
use rocket::local::Client;

use eloquentlog_backend_api::app;
use eloquentlog_backend_api::config::Config;
use eloquentlog_backend_api::model::message;

use run_test;

#[test]
fn test_get() {
    run_test(|| {
        let client = Client::new(app("testing")).unwrap();
        let mut res = client.get("/api/messages").dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("[]"));
    })
}

#[test]
fn test_post_with_errors() {
    run_test(|| {
        let client = Client::new(app("testing")).unwrap();
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
    run_test(|| {
        let client = Client::new(app("testing")).unwrap();
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
    run_test(|| {
        let c = Config::from("testing").unwrap();
        let conn =
            PgConnection::establish(&c.database_url).unwrap_or_else(|_| {
                panic!("Error connecting to : {}", c.database_url)
            });

        let m = message::NewMessage {
            code: None,
            lang: "en".to_string(),
            level: message::Level::Information,
            format: message::Format::TOML,
            title: Some("title".to_string()),
            content: None,
        };

        let id = diesel::insert_into(message::messages::table)
            .values(&m)
            .returning(message::messages::id)
            .get_result::<i64>(&conn)
            .unwrap_or_else(|_| panic!("Error inserting: {}", m));

        let client = Client::new(app("testing")).unwrap();

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
            .first::<message::Message>(&conn)
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
