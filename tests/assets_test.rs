extern crate rocket;
extern crate dotenv;

#[cfg(test)]
mod assets_test {
    extern crate montafon;

    use std::panic;

    use dotenv::dotenv;
    use rocket::local::Client;
    use rocket::http::Status;

    fn run_test<T>(test: T) -> ()
        where T: FnOnce() -> () + panic::UnwindSafe {
        setup();
        let result = panic::catch_unwind(|| {
            test()
        });
        teardown();
        assert!(result.is_ok())
    }

    fn setup() {
        dotenv().ok();
    }

    fn teardown() {
    }

    #[test]
    fn test_static_style_css() {
        run_test(|| {
            let client = Client::new(montafon::app("testing")).unwrap();
            let mut res = client.get("/static/css/style.css").dispatch();
            assert_eq!(res.status(), Status::Ok);
            assert!(res.body_string().unwrap().contains("h1"));
        })
    }
}