extern crate rocket;

#[cfg(test)]
mod top_test {
    extern crate montafon;

    use rocket::local::Client;
    use rocket::http::Status;

    #[test]
    fn test_static_style_css() {
        let client = Client::new(montafon::app()).unwrap();
        let mut res = client.get("/static/css/style.css").dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(res.body_string().unwrap().contains("h1"));
    }
}
