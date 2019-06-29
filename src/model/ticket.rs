//! Ticket handles encoding/decoding using Claims and Token.
use std::fmt;

use jsonwebtoken::{
    Algorithm, Header, Validation, decode as decode_token, decode_header,
    encode as encode_token,
};

#[derive(Clone)]
pub struct Token {
    pub value: String, // subject

    // timestamp values
    pub granted_at: i64,
    pub expires_at: i64,
}

impl fmt::Display for Token {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.value)?;
        Ok(())
    }
}

pub trait Claims
where Self: std::marker::Sized
{
    const ALGORITHM: Algorithm;
    const LEEWAY: i64;

    fn decode(
        ticket: &str, // encoded string
        issuer: &str,
        secret: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error>;

    fn encode(
        token: Token, // contains subject
        issuer: &str,
        kei_id: &str,
        secret: &str,
    ) -> String;

    fn get_subject(&self) -> String;
}

/// ActivationClaims
#[derive(Debug, Deserialize, Serialize)]
pub struct ActivationClaims {
    pub sub: String,
    pub iat: usize,
    pub iss: String,
    pub exp: usize,
    pub nbf: usize,
}

impl Claims for ActivationClaims {
    const ALGORITHM: Algorithm = Algorithm::HS512;
    const LEEWAY: i64 = 36; // seconds

    fn decode(
        ticket: &str,
        issuer: &str,
        secret: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error>
    {
        // self check
        let header = decode_header(&ticket).expect("Invalid token");
        if header.alg != Self::ALGORITHM {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }

        // TODO: validate aud
        let v = Validation {
            algorithms: vec![Self::ALGORITHM],
            iss: Some(issuer.to_string()),
            leeway: Self::LEEWAY,
            validate_exp: true,
            validate_nbf: true,

            ..Validation::default()
        };

        match decode_token::<Self>(&ticket, secret.as_ref(), &v) {
            Ok(v) => Ok(v.claims),
            Err(e) => Err(e),
        }
    }

    fn encode(
        token: Token,
        issuer: &str,
        key_id: &str,
        secret: &str,
    ) -> String
    {
        // TODO: aud
        let c = Self {
            sub: token.value,
            iat: token.granted_at as usize,
            iss: issuer.to_string(),
            exp: token.expires_at as usize,
            nbf: token.granted_at as usize,
        };

        let mut h = Header::default();
        h.alg = Self::ALGORITHM;
        h.kid = Some(key_id.to_string());

        encode_token(&h, &c, secret.as_ref()).unwrap()
    }

    fn get_subject(&self) -> String {
        self.sub.to_string()
    }
}

/// AuthorizationClaims
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthorizationClaims {
    pub sub: String,
    pub iat: usize,
    pub iss: String,
    pub exp: usize,
    pub nbf: usize,
}

impl Claims for AuthorizationClaims {
    const ALGORITHM: Algorithm = Algorithm::HS256;
    const LEEWAY: i64 = 36; // seconds

    fn decode(
        ticket: &str,
        issuer: &str,
        secret: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error>
    {
        // self check
        let header = decode_header(&ticket).expect("Invalid token");
        if header.alg != Self::ALGORITHM {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }

        // TODO: validate aud
        let v = Validation {
            algorithms: vec![Self::ALGORITHM],
            iss: Some(issuer.to_string()),
            leeway: Self::LEEWAY,
            validate_exp: false,
            validate_nbf: true,

            ..Validation::default()
        };

        match decode_token::<Self>(&ticket, secret.as_ref(), &v) {
            Ok(v) => Ok(v.claims),
            Err(e) => Err(e),
        }
    }

    fn encode(
        token: Token,
        issuer: &str,
        key_id: &str,
        secret: &str,
    ) -> String
    {
        // TODO: aud
        let c = Self {
            sub: token.value,
            iat: token.granted_at as usize,
            iss: issuer.to_string(),
            exp: token.expires_at as usize,
            nbf: token.granted_at as usize,
        };

        let mut h = Header::default();
        h.alg = Self::ALGORITHM;
        h.kid = Some(key_id.to_string());

        encode_token(&h, &c, secret.as_ref()).unwrap()
    }

    fn get_subject(&self) -> String {
        self.sub.to_string()
    }
}

#[cfg(test)]
mod ticket_test {
    use super::*;

    extern crate base64;
    use self::base64::decode;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use serde_json;

    use model::test::run;

    #[test]
    fn test_token_format() {
        let now = Utc::now();
        let ts = now.timestamp();

        let t = Token {
            value: "dummy".to_string(),
            granted_at: ts,
            expires_at: ts,
        };

        assert_eq!(format!("{}", t), "dummy");
    }

    #[test]
    fn test_activation_claims_encode() {
        let now = Utc.ymd(2019, 6, 11).and_hms(23, 19, 32);
        let token = Token {
            value: "dummy".to_string(),
            granted_at: now.timestamp(),
            expires_at: (now + Duration::hours(24)).timestamp(),
        };

        let ticket = ActivationClaims::encode(
            token.clone(),
            "issuer",
            "key_id",
            "secret",
        );

        assert_eq!(
            ticket,
            [
                "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiIsImtpZCI6ImtleV9pZCJ9",
                concat!(
                    "eyJzdWIiOiJkdW1teSIsImlhdCI6MTU2MDI5NTE3MiwiaXNzIjoiaXNzdW",
                    "VyIiwiZXhwIjoxNTYwMzgxNTcyLCJuYmYiOjE1NjAyOTUxNzJ9",
                ),
                concat!(
                    "sEQtl1gRn3q5YwdYboRQ9sh0YbmmzL62_wMRRbOSurHHtUFJTccPk_-YhZ",
                    "v_X8XNx0jhg9ebUUR7BYS9iHjIww",
                )
            ].join(".")
        );

        let s: Vec<&str> = ticket.split('.').collect();
        let body = &decode(s[1]).unwrap()[..]; // base64
        let json = String::from_utf8_lossy(body).to_string();

        let claims: ActivationClaims =
            serde_json::from_str(&json).ok().unwrap();
        assert_eq!(claims.sub, token.value);
        assert_eq!(claims.iss, "issuer");
        assert_eq!(claims.iat, 1_560_295_172);
        assert_eq!(claims.exp, claims.iat + 60 * 60 * 24); // +86400 (1560381572)
        assert_eq!(claims.nbf, 1_560_295_172);
    }

    #[test]
    fn test_activation_claims_decode_failure() {
        run(|_, config, _| {
            let tests: [(&str, &str, &str, DateTime<Utc>); 4] = [
                (
                    // expires
                    "dummy",
                    &config.activation_ticket_issuer,
                    &config.activation_ticket_secret,
                    Utc.ymd(2001, 1, 1).and_hms(10, 0, 0),
                ),
                (
                    // not before
                    "dummy",
                    &config.activation_ticket_issuer,
                    &config.activation_ticket_secret,
                    Utc::now() + Duration::hours(3),
                ),
                (
                    // wrong issuer
                    "dummy",
                    "unknown",
                    &config.activation_ticket_secret,
                    Utc::now(),
                ),
                (
                    // invalid secret
                    "dummy",
                    &config.activation_ticket_issuer,
                    "invalid",
                    Utc::now(),
                ),
            ];
            for (_, (value, issuer, secret, granted_at)) in
                tests.iter().enumerate()
            {
                let token = Token {
                    value: value.to_string(),
                    granted_at: granted_at.timestamp(),
                    expires_at: (*granted_at + Duration::hours(24)).timestamp(),
                };
                let ticket = ActivationClaims::encode(
                    token,
                    &config.activation_ticket_issuer,
                    &config.activation_ticket_key_id,
                    &config.activation_ticket_secret,
                );
                assert!(
                    ActivationClaims::decode(&ticket, issuer, secret).is_err()
                );
            }
        });
    }

    #[test]
    fn test_activation_claims_decode() {
        let granted_at = Utc::now();
        let token = Token {
            value: "dummy".to_string(),
            granted_at: granted_at.timestamp(),
            expires_at: (granted_at + Duration::hours(24)).timestamp(),
        };

        let ticket = ActivationClaims::encode(
            token.clone(),
            "issuer",
            "key_id",
            "secret",
        );

        let claims = ActivationClaims::decode(&ticket, "issuer", "secret")
            .ok()
            .unwrap();

        assert_eq!(claims.sub, token.value);
        assert_eq!(claims.iss, "issuer");
        assert_eq!(claims.iat, token.granted_at as usize);
        assert_eq!(claims.exp, claims.iat + 60 * 60 * 24); // +86400 (1560381572)
        assert_eq!(claims.nbf, token.granted_at as usize);
    }
}
