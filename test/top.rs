use rocket::http::Status;

use run_test;

#[test]
fn test_index() {
    run_test(|client, _, _, _| {
        let mut res = client.get("/").dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("Eloquentlog"));
    });
}
