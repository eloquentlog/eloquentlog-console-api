use lettre_email::Email;
use slog::Logger;

use config::Config;
use mailer::{Header, Mailer};

pub struct UserMailer<'a> {
    config: &'a Config,
    header: Header<'a>,
    mailer: Mailer<'a>,
}

impl<'a> UserMailer<'a> {
    pub fn new(config: &'a Config, logger: &'a Logger) -> Self {
        let header = Header {
            from: (&config.mailer_from_email, &config.mailer_from_alias),

            ..Default::default()
        };
        let mailer = Mailer::new(config, logger);

        Self {
            config,
            header,
            mailer,
        }
    }

    pub fn to(&mut self, to: (&'a str, &'a str)) -> &mut Self {
        self.header.to = to;
        self
    }

    pub fn send_user_activation_email(&mut self, token: String) -> bool {
        let url = self.config.application_url.to_string();
        // TODO: build it with rocket::http::uri::Origin?
        let activation_url = format!("{}/user/activate?token={}", url, token);

        let subject = "Activate your account";
        // TODO: use template file
        let message = format!(
            r#"
Welcome to Eloquentlog!

You have successfully signed up to Eloquentlog.
To activate your account, just follow the link below

{}

Happy logging !-)

--
Eloquentlog
{}
"#,
            activation_url, url,
        );
        let email = Email::builder()
            .to(self.header.to)
            .from(self.header.from)
            .subject(subject)
            .text(message)
            .build()
            .unwrap();
        self.mailer.send(email.into())
    }
}
