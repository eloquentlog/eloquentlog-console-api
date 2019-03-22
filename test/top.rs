use rocket::http::Status;
use rocket::local::Client;

use run_test;

#[test]
fn test_index() {
    run_test(|| {
        let client =
            Client::new(eloquentlog_backend_api::app("testing")).unwrap();
        let mut res = client.get("/").dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("Eloquentlog"));
    })
}
