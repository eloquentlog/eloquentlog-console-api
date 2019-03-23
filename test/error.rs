use rocket::http::Status;

use run_test;

#[test]
fn test_404_not_found() {
    run_test(|client, _| {
        let mut res = client.get("/unknown-path").dispatch();
        assert_eq!(res.status(), Status::NotFound);
        assert!(res.body_string().unwrap().contains("Not Found"));
    })
}
