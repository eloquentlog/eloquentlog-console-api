use slog::Logger;

use config::Config;
use mailer::Mailer;

pub struct UserMailer<'a> {
    mailer: Mailer<'a>,
}

impl<'a> UserMailer<'a> {
    pub fn new(config: &'a Config, logger: &'a Logger) -> Self {
        let mailer = Mailer::new(config, logger);
        Self { mailer }
    }

    pub fn to(&mut self, to: &'a str) -> &mut Self {
        self.mailer.to(to);
        self
    }

    pub fn send_user_activation_email(&mut self, token: String) -> bool {
        // TODO
        let payload = format!("token: {}", token);
        self.mailer.send(payload)
    }
}
