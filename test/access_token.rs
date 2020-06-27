use diesel::{self, prelude::*};
use chrono::{Utc, TimeZone};
use rocket::http::{ContentType, Header, Status};
use serde_json::Value;
use uuid::Uuid;

use eloquentlog_console_api::model;

use crate::{minify, run_test, load_user, make_raw_password, USERS};

#[test]
fn test_access_token_hset_state_failure() {
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

        let state = model::access_token::AccessTokenState::Enabled;
        let res = client
            .patch(format!("/_api/access_token/hset/{}/state", Uuid::nil()))
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .body(format!(
                r#"{{
                  "access_token": {{
                    "state": "{}"
                  }}
                }}"#,
                &state,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::NotFound);
    });
}

#[test]
fn test_access_token_hset_state() {
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

        let v = model::access_token::AccessToken::generate_token();
        let t = model::access_token::AccessToken {
            id: 1,
            uuid: Uuid::new_v4(),
            agent_id: user.id,
            agent_type: model::access_token::AgentType::Person,
            name: "personal token".to_string(),
            token: Some(v.into_bytes()),
            state: model::access_token::AccessTokenState::Disabled,
            revoked_at: None,
            created_at: dt.naive_utc(),
            updated_at: dt.naive_utc(),
        };

        let access_token =
            diesel::insert_into(model::access_token::access_tokens::table)
                .values(&t)
                .get_result::<model::access_token::AccessToken>(conn.db)
                .unwrap_or_else(|_| panic!("Error inserting: {}", t));

        let state = model::access_token::AccessTokenState::Enabled;
        let mut res = client
            .patch(format!(
                "/_api/access_token/hset/{}/state",
                access_token.uuid,
            ))
            .header(ContentType::JSON)
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .body(format!(
                r#"{{
                  "access_token": {{
                    "state": "{}"
                  }}
                }}"#,
                &state,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let body = res.body_string().unwrap();
        assert_eq!(body, "{\"access_token\":1}");
    });
}

#[test]
fn test_access_token_lrange_returns_empty_if_not_exist() {
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
            .get("/_api/access_token/lrange/client/0/0")
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let body = res.body_string().unwrap();
        assert_eq!(body, "[]");
    });
}

#[test]
fn test_access_token_lrange_returns_a_list_contain_tokens() {
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
        let v = model::access_token::AccessToken::generate_token();
        let t = model::access_token::AccessToken {
            id: 1,
            uuid: Uuid::new_v4(),
            agent_id: user.id,
            agent_type: model::access_token::AgentType::Client,
            name: "client token 1".to_string(),
            token: Some(v.into_bytes()),
            state: model::access_token::AccessTokenState::Enabled,
            revoked_at: None,
            created_at: dt.naive_utc(),
            updated_at: dt.naive_utc(),
        };

        let access_token_1 =
            diesel::insert_into(model::access_token::access_tokens::table)
                .values(&t)
                .get_result::<model::access_token::AccessToken>(conn.db)
                .unwrap_or_else(|_| panic!("Error inserting: {}", t));

        let dt = Utc.ymd(2020, 2, 18).and_hms_milli(5, 4, 3, 222);
        let v = model::access_token::AccessToken::generate_token();
        let t = model::access_token::AccessToken {
            id: 2,
            uuid: Uuid::new_v4(),
            agent_id: user.id,
            agent_type: model::access_token::AgentType::Client,
            name: "client token 2".to_string(),
            token: Some(v.into_bytes()),
            state: model::access_token::AccessTokenState::Enabled,
            revoked_at: None,
            created_at: dt.naive_utc(),
            updated_at: dt.naive_utc(),
        };

        let access_token_2 =
            diesel::insert_into(model::access_token::access_tokens::table)
                .values(&t)
                .get_result::<model::access_token::AccessToken>(conn.db)
                .unwrap_or_else(|e| {
                    dbg!(e);
                    panic!("");
                });

        let mut res = client
            .get("/_api/access_token/lrange/client/0/1")
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .header(Header::new("X-Requested-With", "XMLHttpRequest"))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);

        let body = res.body_string().unwrap();
        assert_eq!(
            body,
            minify(format!(
                r#"[{{
"access_token": {{
  "agent_type": "client",
  "created_at": "2019-08-07T06:05:04.333",
  "name": "client token 1",
  "revoked_at": null,
  "state": "enabled",
  "token": "***",
  "updated_at": "2019-08-07T06:05:04.333",
  "uuid": "{}"
}}}},{{
"access_token": {{
  "agent_type": "client",
  "created_at": "2020-02-18T05:04:03.222",
  "name": "client token 2",
  "revoked_at": null,
  "state": "enabled",
  "token": "***",
  "updated_at": "2020-02-18T05:04:03.222",
  "uuid": "{}"
}}}}]"#,
                access_token_1.uuid, access_token_2.uuid,
            ))
        );
    });
}
