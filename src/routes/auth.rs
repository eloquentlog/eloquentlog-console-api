use std::collections::HashMap;

use regex::Regex;

use rocket_contrib::Template;
use rocket::http::RawStr;
use rocket::request::{FlashMessage, Form, FromFormValue};

use db::DbConn;


#[derive(Debug)]
struct ValidUsername<'r>(&'r str);

#[derive(Debug)]
struct ValidPassword<'r>(&'r str);

#[derive(FromForm)]
struct UserLogin<'r> {
    username: Result<ValidUsername<'r>, &'static str>,
    password: Result<ValidPassword<'r>, &'static str>,
}

impl<'v> FromFormValue<'v> for ValidUsername<'v> {
    type Error = &'static str;

    fn from_form_value(v: &'v RawStr) -> Result<Self, Self::Error> {
        let decoded_str = match v.percent_decode() {
            Ok(s) => s.to_string(),
            Err(_) => return Err("invalid input"),
        };
        // required
        if decoded_str == "" {
            return Err("required");
        }
        // simple check as email
        match decoded_str.contains("@") && decoded_str.contains(".") {
            true => Ok(ValidUsername(v.as_str())),
            false => Err("wrong format"),
        }
    }
}


impl<'v> FromFormValue<'v> for ValidPassword<'v> {
    type Error = &'static str;

    fn from_form_value(v: &'v RawStr) -> Result<Self, Self::Error> {
        let decoded_str = match v.percent_decode() {
            Ok(s) => s.to_string(),
            Err(_) => return Err("invalid input"),
        };
        // required
        if decoded_str == "" {
            return Err("required");
        }
        // length
        if decoded_str.len() < 8 {
            return Err("too short");
        }
        // format
        lazy_static! {
            static ref RE: Regex = Regex::new(r"[A-z_\-\d]+").unwrap();
        }
        match RE.is_match(&decoded_str) {
            false => Err("wrong format"),
            true  => Ok(ValidPassword(v.as_str())),
        }
    }
}

#[derive(Serialize)]
struct FormData {
    data: HashMap<&'static str, String>,
    errors: Option<HashMap<&'static str, String>>,
}

#[derive(Serialize)]
struct TemplateContext {
    title: &'static str,
    flash: Option<String>,
    form: FormData,
}

fn validate_user_login(user: &UserLogin) -> HashMap<&'static str, String> {
    let mut errors = HashMap::new();

    if let Err(e) = user.username {
        errors.insert("username",
            format!("Username is invalid: {}", e).to_string());
    }
    if let Err(e) = user.password {
        errors.insert("password",
            format!("Password is invalid: {}", e).to_string());
    }
    errors
}

fn value_of(v: &RawStr) -> String {
    match v.percent_decode() {
        Ok(s) => s.to_string(),
        Err(_) => "".to_string(),
    }
}

// route actions

#[get("/login")]
fn login_get(flash: Option<FlashMessage>) -> Template {
    let mut ctx = HashMap::new();
    ctx.insert("title", "Eloquentlog;)");
    if let Some(ref msg) = flash {
        ctx.insert("flash", msg.msg());
    }
    Template::render("auth/login", &ctx)
}


#[post("/login", data = "<user_form>")]
fn login_post<'a>(
    _conn: DbConn, user_form: Form<'a, UserLogin<'a>>) -> Template {

    let user = user_form.get();
    let errors = validate_user_login(user);

    let mut flash = None;
    if errors.is_empty() {
        // TODO: check user
    }

    if !errors.is_empty() {
        flash = Some(
            "The credentials you've entered is incorrect.".to_string());
    }

    // TODO: more efficient way?
    let mut data = HashMap::new();
    data.insert("username", match user.username {
        Ok(ref v) => value_of(RawStr::from_str(v.0)),
        Err(_) => "".to_string(),
    });

    let form = FormData {
        data: data,
        errors: Some(errors),
    };

    let ctx = TemplateContext {
        title: "Eloquentlog;)",
        form: form,
        flash: flash,
    };
    Template::render("auth/login", &ctx)
}
