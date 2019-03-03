extern crate rocket;
extern crate dotenv;

#[cfg(test)]
mod messages_test {
    extern crate eloquentlog_backend_api;

    use std::panic;

    use dotenv::dotenv;
    use rocket::http::{ContentType, Status};
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
    fn test_get() {
        run_test(|| {
            let client =
                Client::new(eloquentlog_backend_api::app("testing")).unwrap();
            let mut res = client.get("/api/messages").dispatch();

            assert_eq!(res.status(), Status::Ok);
            assert!(res.body_string().unwrap().contains("[]"));
        })
    }

    #[test]
    fn test_post() {
        run_test(|| {
            let client =
                Client::new(eloquentlog_backend_api::app("testing")).unwrap();
            let mut res = client
                .post("/api/messages")
                .header(ContentType::JSON)
                .body(
                    r#"{
            "format": "toml",
            "title": "New message",
            "content": "Hello, world!"
          }"#,
                )
                .dispatch();

            assert_eq!(res.status(), Status::Ok);
            assert!(res.body_string().unwrap().contains("id"));
        })
    }

    #[test]
    fn test_put() {
        run_test(|| {
            let client =
                Client::new(eloquentlog_backend_api::app("testing")).unwrap();
            let mut res = client
                .put("/api/messages/3")
                .header(ContentType::JSON)
                .body(
                    r#"{
            "id": 3,
            "title": "Updated message",
            "content": "Hello, world!"
          }"#,
                )
                .dispatch();

            assert_eq!(res.status(), Status::Ok);
            assert!(res.body_string().unwrap().contains("\"id\":3"));
        })
    }
}
