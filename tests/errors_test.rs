extern crate rocket;

#[cfg(test)]
mod errors_test {
    extern crate montafon;

    use rocket::local::Client;
    use rocket::http::Status;

    #[test]
    fn test_404_not_found() {
        let client = Client::new(montafon::app()).unwrap();
        let mut res = client.get("/unknown-path").dispatch();
        assert_eq!(res.status(), Status::NotFound);
        assert!(res.body_string().unwrap().contains("Not Found"));
    }
}
