extern crate rocket;
extern crate dotenv;

#[cfg(test)]
mod login_test {
    extern crate eloquentlog_backend_api;

    use std::panic;

    use dotenv::dotenv;
    use rocket::local::Client;
    use rocket::http::{ContentType, Status};

    fn run_test<T>(test: T)
    where T: FnOnce() -> () + panic::UnwindSafe {
        setup();
        let result = panic::catch_unwind(test);
        teardown();
        assert!(result.is_ok())
    }

    fn setup() {
        dotenv().ok();
    }

    fn teardown() {}

    #[test]
    fn test_login() {
        run_test(|| {
            let client =
                Client::new(eloquentlog_backend_api::app("testing")).unwrap();
            let req = client.post("/login").header(ContentType::JSON).body(
                "{\"username\": \"u$ername\", \"password\": \"pa$$w0rd\"}",
            );
            let mut res = req.dispatch();

            assert_eq!(res.status(), Status::Ok);
            assert!(res.body_string().unwrap().contains("Success"));
        })
    }
}
