pub mod user;

use lettre::{
    ClientSecurity, ClientTlsParameters, EmailAddress, Envelope, Transport,
    SendableEmail, SmtpClient,
};
use lettre::smtp::ConnectionReuseParameters;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::error::SmtpResult;
use lettre::smtp::extension::ClientId;
use lettre::smtp::client::net::DEFAULT_TLS_PROTOCOLS;
use native_tls::TlsConnector;
use slog::Logger;
use uuid::Uuid;

use config::Config;

struct Header<'a> {
    id: Uuid,
    from: &'a str,
    to: &'a str,
}

impl<'a> Default for Header<'a> {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            from: "",
            to: "",
        }
    }
}

type Client<'a> = Box<dyn Transport<'a, Result = SmtpResult>>;

pub struct Mailer<'a> {
    header: Header<'a>,
    client: Option<Client<'a>>,
    config: &'a Config,
    logger: &'a Logger,
}

impl<'a> Mailer<'a> {
    // TODO: connection manager (r2d2)
    pub fn build_client(config: &Config) -> Client<'a> {
        let mut tls_builder = TlsConnector::builder();
        tls_builder.min_protocol_version(Some(DEFAULT_TLS_PROTOCOLS[0]));
        let tls_parameters = ClientTlsParameters::new(
            config.mailer_smtp_host.to_string(),
            tls_builder.build().unwrap(),
        );
        // with custom port
        let client = SmtpClient::new(
            (config.mailer_smtp_host.as_str(), config.mailer_smtp_port),
            ClientSecurity::Wrapper(tls_parameters),
        )
        .unwrap()
        .hello_name(ClientId::Domain(config.mailer_domain.to_string()))
        .credentials(Credentials::new(
            config.mailer_smtp_username.to_string(),
            config.mailer_smtp_password.to_string(),
        ))
        .smtp_utf8(true)
        .authentication_mechanism(Mechanism::Plain)
        .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
        .transport();
        Box::new(client)
    }

    pub fn new(config: &'a Config, logger: &'a Logger) -> Self {
        let header = Header {
            ..Default::default()
        };
        let client = None;

        Self {
            header,
            client,
            config,
            logger,
        }
    }

    pub fn inject(&mut self, client: Option<Client<'a>>) {
        self.client = client;
    }

    pub fn to(&mut self, to: &'a str) -> &mut Self {
        self.header = Header {
            id: Uuid::new_v4(),
            from: &self.config.mailer_from_email,
            to,
        };
        self
    }

    pub fn send(&mut self, payload: String) -> bool {
        let email = SendableEmail::new(
            // TODO: sender alias
            Envelope::new(
                Some(EmailAddress::new(self.header.from.to_string()).unwrap()),
                vec![EmailAddress::new(self.header.to.to_string()).unwrap()],
            )
            .unwrap(),
            self.header.id.to_urn().to_string(),
            payload.into_bytes(),
        );

        let result;
        if let Some(ref mut c) = self.client {
            result = c.send(email);
        } else {
            let mut client = Self::build_client(self.config);
            result = client.send(email);
        }
        if let Err(ref e) = result {
            error!(self.logger, "e: {}", e);
        }
        result.is_ok()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use lettre::smtp::response::{
        Category, Code, Detail, Response as SmtpResponse, Severity,
    };

    use model::test::run;
    use model::user::data::USERS;

    // NOTE:
    // Apparently, in v0.9.2, it seems that `StubTransport` holds `StubResult`
    // as a response. It makes us hard to replace an transport instance while
    // testing in our usage. Thus, we simply use a mocked transport with
    // SmtpResponse.
    //
    // https://docs.rs/lettre/0.9.2/src/lettre/stub/mod.rs.html#11
    struct MockTransport {
        response: SmtpResponse,
    }

    impl<'a> Transport<'a> for MockTransport {
        type Result = SmtpResult;

        fn send(&mut self, _email: SendableEmail) -> SmtpResult {
            let response = self.response.clone();
            if response.is_positive() {
                Ok(response)
            } else {
                Err(response.into())
            }
        }
    }

    #[test]
    fn test_email_send_failure() {
        run(|_, config, logger| {
            let mut mailer = Mailer::new(config, logger);

            let code = Code::new(
                Severity::TransientNegativeCompletion,
                Category::Connections,
                Detail::Zero,
            );
            let response = SmtpResponse {
                code,
                message: vec![],
            };
            let transport = MockTransport { response };
            mailer.inject(Some(Box::new(transport)));

            let u = USERS.get("oswald").unwrap();
            assert!(!mailer.to(&u.email).send("Hello, world!".to_string()));
        })
    }

    #[test]
    fn test_email_send_success() {
        run(|_, config, logger| {
            let mut mailer = Mailer::new(config, logger);

            let code = Code::new(
                Severity::PositiveCompletion,
                Category::MailSystem,
                Detail::Zero,
            );
            let response = SmtpResponse {
                code,
                message: vec![],
            };
            let transport = MockTransport { response };
            mailer.inject(Some(Box::new(transport)));

            let u = USERS.get("oswald").unwrap();
            assert!(mailer.to(&u.email).send("Hello, world!".to_string()));
        })
    }
}
