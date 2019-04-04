use rocket::http::{ContentType, Status};

use run_test;

#[test]
fn test_login() {
    run_test(|client, _| {
        let req = client
            .post("/_api/login")
            .header(ContentType::JSON)
            .body("{\"username\": \"u$ername\", \"password\": \"pa$$w0rd\"}");
        let mut res = req.dispatch();

        assert_eq!(res.status(), Status::Unauthorized);
        assert_eq!("null", res.body_string().unwrap());
    });
}
