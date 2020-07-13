use diesel::{self, prelude::*};
use rocket::http::{ContentType, Header, Status};
use serde_json::Value;

use eloquentlog_console_api::model;

use crate::{
    minify, run_test, load_user, make_raw_password, MEMBERSHIPS, NAMESPACES,
    USERS,
};

#[test]
fn test_hgetall_no_namespace() {
    run_test(|client, conn, _, _| {
        let u = USERS.get("oswald").unwrap().clone();
        let password = make_raw_password(&u);
        let user = load_user(u, conn.db);

        let _ = client
            .head("/_/login/")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let mut res = client
            .post("/_/login")
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
            .get("/v1/namespace/hgetall")
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("[]"));
    });
}

#[test]
fn test_hgetall_namespaces() {
    run_test(|client, conn, _, _| {
        let u = USERS.get("oswald").unwrap().clone();
        let password = make_raw_password(&u);
        let user = load_user(u, conn.db);

        let _ = client
            .head("/_/login/")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body("{}")
            .dispatch();

        let mut res = client
            .post("/_/login")
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
        let namespace =
            diesel::insert_into(model::namespace::namespaces::table)
                .values(ns)
                .get_result::<model::namespace::Namespace>(conn.db)
                .unwrap_or_else(|_| panic!("Error inserting: {}", ns));

        let mut ms = MEMBERSHIPS
            .get("oswald as a primary owner")
            .unwrap()
            .clone();
        ms.namespace_id = namespace.id;
        let _ = diesel::insert_into(model::membership::memberships::table)
            .values(&ms)
            .returning(model::membership::memberships::id)
            .get_result::<i64>(conn.db)
            .unwrap_or_else(|_| panic!("Error inserting: {}", ms));

        let mut res = client
            .get("/v1/namespace/hgetall")
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        assert_eq!(
            res.body_string().unwrap(),
            minify(format!(
                r#"[{{
"namespace": {{
  "archived_at": null,
  "created_at": "2019-07-07T07:20:15",
  "description": "description",
  "name": "piano",
  "streams_count": 0,
  "updated_at": "2019-07-07T07:20:15",
  "uuid": "{}"
}}
}}]"#,
                namespace.uuid,
            ))
        );
    });
}
