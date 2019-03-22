use rocket::http::Status;
use rocket::local::Client;

use run_test;

#[test]
fn test_404_not_found() {
    run_test(|| {
        let client =
            Client::new(eloquentlog_backend_api::app("testing")).unwrap();
        let mut res = client.get("/unknown-path").dispatch();
        assert_eq!(res.status(), Status::NotFound);
        assert!(res.body_string().unwrap().contains("Not Found"));
    })
}
