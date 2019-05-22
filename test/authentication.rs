use rocket::http::{ContentType, Status};
use diesel::{self, prelude::*};

use eloquentlog_backend_api::model::user;

use run_test;

#[test]
fn test_login() {
    run_test(|client, conn| {
        let password = "pa$$w0rD";
        let mut u = user::NewUser {
            name: None,
            username: None,
            email: "foo@example.org".to_string(),

            ..Default::default()
        };
        u.set_password(&password);

        let _id = diesel::insert_into(user::users::table)
            .values(&u)
            .returning(user::users::id)
            .get_result::<i64>(conn)
            .unwrap_or_else(|_| panic!("Error inserting: {}", u));

        let req =
            client
                .post("/_api/login")
                .header(ContentType::JSON)
                .body(format!(
                    "{{\"username\": \"{}\", \"password\": \"{}\"}}",
                    u.email, password
                ));
        let res = req.dispatch();

        assert_eq!(res.status(), Status::Ok);
    });
}
