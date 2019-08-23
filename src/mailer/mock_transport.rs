use lettre::{SendableEmail, Transport};
use lettre::smtp::error::SmtpResult;
use lettre::smtp::response::Response;

/// A mock implements Transport for doctest and test
struct MockTransport {
    // NOTE:
    // Apparently, in v0.9.2, it seems that `StubTransport` holds `StubResult`
    // as a response. It makes us hard to replace an transport instance while
    // testing in our usage. Thus, we simply use a mocked transport with
    // SmtpResponse.
    //
    // https://docs.rs/lettre/0.9.2/src/lettre/stub/mod.rs.html#11
    response: Response,
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

impl MockTransport {
    fn new(code: Code, message: Vec<String>) -> Self {
        let response = Response {
            code,
            message,
        };
        MockTransport { response }
    }
}
