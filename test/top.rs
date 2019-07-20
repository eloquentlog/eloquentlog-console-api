use rocket::http::Status;

use {run_test, build_authorization_header, load_user, USERS};

#[test]
fn test_index() {
    run_test(|client, db_conn, _, config, _| {
        let user = load_user(&USERS.get("oswald").unwrap(), db_conn);
        let auth = build_authorization_header(&user, config);
        let mut res = client.get("/").header(auth).dispatch();

        assert_eq!(res.status(), Status::Ok);

        let body = res.body_string().unwrap();
        assert!(body.contains(user.username.as_str()));
        assert!(body.contains("Eloquentlog"));
    });
}
