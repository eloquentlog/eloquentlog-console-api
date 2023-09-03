use rocket::http::{ContentType, Header, Status};
use serde_json::Value;

use crate::{run_test, load_user, make_raw_password, USERS};

#[test]
fn test_health_check() {
    run_test(|client, _, _, _| {
        let res = client
            .get("/_/health")
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
    });
}

#[test]
fn test_v1_health_check() {
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

        let res = client
            .get("/v1/health")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
    });
}
