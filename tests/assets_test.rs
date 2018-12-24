extern crate rocket;
extern crate dotenv;

#[cfg(test)]
mod assets_test {
    extern crate eloquentlog_backend_api;

    use std::panic;

    use dotenv::dotenv;
    use rocket::http::Status;
    use rocket::local::Client;

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
    fn test_static_style_css() {
        run_test(|| {
            let client =
                Client::new(eloquentlog_backend_api::app("testing")).unwrap();
            let mut res = client.get("/static/css/style.css").dispatch();
            assert_eq!(res.status(), Status::Ok);
            assert!(res.body_string().unwrap().contains("h1"));
        })
    }
}
