//! UserMailer

use lettre_email::Email;
use slog::Logger;

use config::Config;
use mailer::{Client, Header, Mailer};

/// UserMailer is a wrapper handles email to user.
///
/// ## Examples
///
/// ```rust
/// # extern crate dotenv;
/// # extern crate lettre;
/// # extern crate lettre_email;
/// #
/// # use dotenv::dotenv;
/// # use lettre::smtp::response::{Category, Code, Detail, Severity};
/// #
/// # use eloquentlog_backend_api::config::Config;
/// # use eloquentlog_backend_api::logger;
/// # use eloquentlog_backend_api::mailer::user::UserMailer;
/// #
/// # include!("./mock_transport.rs");
/// #
/// # fn main() {
/// #     dotenv().ok();
/// #
/// #     let code = Code::new(
/// #         Severity::PositiveCompletion,
/// #         Category::MailSystem,
/// #         Detail::Zero,
/// #     );
/// #     let transport = MockTransport::new(code, vec![]);
/// #
/// let config = Config::from("testing").unwrap();
/// let logger = logger::get_logger(&config);
///
/// let t = "...";
/// let s = "...";
///
/// // you need to initialize it as mut
/// let mut mailer = UserMailer::new(&config, &logger);
/// # mailer.inject(Some(Box::new(transport)));
/// let result = mailer
///     .to(("postmaster@eloquentlog.com", "Name"))
///     .send_user_activation_email(t, s);
/// assert!(result);
/// #
/// # }
/// ```
pub struct UserMailer<'a> {
    /// Config object.
    config: &'a Config,
    /// Email Header object.
    header: Header<'a>,
    /// Mailer is the actual mailer holds client.
    mailer: Mailer<'a>,
}

impl<'a> UserMailer<'a> {
    /// Creates a new UserMailer.
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

    /// Sets to and returns mailer itself.
    pub fn to(&mut self, to: (&'a str, &'a str)) -> &mut Self {
        self.header.to = to;
        self
    }

    pub fn inject(&mut self, client: Option<Client<'a>>) {
        self.mailer.client = client;
    }

    /// Builds an user activation message and send it via actual mailer.
    pub fn send_user_activation_email(&mut self, t: &str, s: &str) -> bool {
        let url = self.config.application_url.to_string();
        // TODO: build it with rocket::http::uri::Origin?
        let activation_url = format!("{}/user/activate?t={}?s={}", url, t, s);

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
