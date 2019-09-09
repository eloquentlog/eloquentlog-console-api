use chrono::{Duration, Utc};
use rocket::http::{ContentType, Status};

use eloquentlog_backend_api::model;
use eloquentlog_backend_api::model::token::Claims;

use run_test;

#[test]
fn test_activate_with_invalid_token() {
    run_test(|client, db_conn, _, config, logger| {
        let mut u = model::user::NewUser {
            name: None,
            username: "johnny".to_string(),
            email: "johnny@example.org".to_string(),

            ..Default::default()
        };
        u.set_password(&"pa$$w0rD");

        let user = model::user::User::insert(&u, &db_conn, &logger)
            .unwrap_or_else(|| panic!("error: {}", u));

        let ue: model::user_email::NewUserEmail = (&user).into();

        let user_email =
            model::user_email::UserEmail::insert(&ue, &db_conn, &logger)
                .unwrap_or_else(|| panic!("error: {}", ue.email));

        let now = Utc::now();
        let data = model::token::TokenData {
            value: model::user_email::UserEmail::generate_token(),
            granted_at: now.timestamp(),
            expires_at: (now + Duration::hours(1)).timestamp(),
        };
        let token = model::token::ActivationClaims::encode(
            data,
            &config.activation_token_issuer,
            &config.activation_token_key_id,
            &config.activation_token_secret,
        );
        let _ = user_email
            .grant_token::<model::token::ActivationClaims>(
                &token,
                &config.activation_token_issuer,
                &config.activation_token_secret,
                &db_conn,
                &logger,
            )
            .unwrap();

        let res = client
            .put("/_api/user/activate")
            .header(ContentType::JSON)
            .body(
                r#"{
                    "token": "invalid-token"
                }"#
                .to_string(),
            )
            .dispatch();

        assert_eq!(res.status(), Status::BadRequest);
        assert!(model::user_email::UserEmail::find_by_token::<
            model::token::ActivationClaims,
        >(
            &token,
            &config.activation_token_issuer,
            &config.activation_token_secret,
            &db_conn,
            &logger
        )
        .is_some());
    });
}

#[test]
fn test_activate() {
    run_test(|client, db_conn, _, config, logger| {
        let mut u = model::user::NewUser {
            name: None,
            username: "johnny".to_string(),
            email: "johnny@example.org".to_string(),

            ..Default::default()
        };
        u.set_password(&"pa$$w0rD");

        let user = model::user::User::insert(&u, &db_conn, &logger)
            .unwrap_or_else(|| panic!("error: {}", u));

        let ue: model::user_email::NewUserEmail = (&user).into();

        let user_email =
            model::user_email::UserEmail::insert(&ue, &db_conn, &logger)
                .unwrap_or_else(|| panic!("error: {}", ue.email));

        let now = Utc::now();
        let data = model::token::TokenData {
            value: model::user_email::UserEmail::generate_token(),
            granted_at: now.timestamp(),
            expires_at: (now + Duration::hours(1)).timestamp(),
        };
        let token = model::token::ActivationClaims::encode(
            data,
            &config.activation_token_issuer,
            &config.activation_token_key_id,
            &config.activation_token_secret,
        );
        let _ = user_email
            .grant_token::<model::token::ActivationClaims>(
                &token,
                &config.activation_token_issuer,
                &config.activation_token_secret,
                &db_conn,
                &logger,
            )
            .unwrap();

        let res = client
            .put("/_api/user/activate")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
          "token": "{}"
        }}"#,
                &token,
            ))
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        assert!(model::user_email::UserEmail::find_by_token::<
            model::token::ActivationClaims,
        >(
            &token,
            &config.activation_token_issuer,
            &config.activation_token_secret,
            &db_conn,
            &logger
        )
        .is_none());
    });
}
