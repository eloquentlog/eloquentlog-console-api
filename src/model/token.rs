//! Token handles encoding/decoding of raw token using Claims.
use std::fmt;

use chrono::{NaiveDateTime, Utc};

use jsonwebtoken::{
    Algorithm, EncodingKey, DecodingKey, Header, Validation,
    decode as decode_token, decode_header, encode as encode_data,
};

use crate::model::user::User;
use crate::model::user_email::UserEmail;

#[derive(Clone)]
pub struct TokenData {
    pub value: String, // subject

    // timestamp values
    pub granted_at: i64,
    pub expires_at: i64,
}

impl fmt::Display for TokenData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.value)?;
        Ok(())
    }
}

impl From<&User> for TokenData {
    fn from(item: &User) -> Self {
        Self {
            value: item.uuid.to_urn().to_string(),
            granted_at: Utc::now().timestamp(),
            expires_at: 0,
        }
    }
}

impl From<&UserEmail> for TokenData {
    fn from(item: &UserEmail) -> Self {
        Self {
            value: item.identification_token.as_ref().unwrap().to_string(),
            granted_at: item
                .identification_token_granted_at
                .unwrap()
                .timestamp(),
            expires_at: item
                .identification_token_expires_at
                .unwrap()
                .timestamp(),
        }
    }
}

pub trait Claims
where Self: std::marker::Sized
{
    const ALGORITHM: Algorithm;
    const LEEWAY: u64;

    fn decode(
        token: &str, // encoded string
        issuer: &str,
        secret: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error>;

    fn encode(
        data: TokenData, // contains subject
        issuer: &str,
        kei_id: &str,
        secret: &str,
    ) -> String;

    fn get_subject(&self) -> String;
    fn get_issued_at(&self) -> NaiveDateTime;
    fn get_expiration_time(&self) -> NaiveDateTime;
}

/// VerificationClaims
///
/// This claims is used for `users.reset_password_token` and
/// `user_emails.identification_token`. They are expected to be treated with an
/// expiration. See also `AccountActivator` service.
#[derive(Debug, Deserialize, Serialize)]
pub struct VerificationClaims {
    pub sub: String,
    pub iat: usize,
    pub iss: String,
    pub exp: usize,
    pub nbf: usize,
}

impl Claims for VerificationClaims {
    const ALGORITHM: Algorithm = Algorithm::HS512;
    const LEEWAY: u64 = 36; // seconds

    fn decode(
        token: &str,
        issuer: &str,
        secret: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error> {
        // self check
        let _ = match decode_header(token) {
            Ok(ref header) if header.alg == Self::ALGORITHM => header,
            _ => {
                return Err(jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::InvalidToken,
                ));
            },
        };

        // TODO: validate aud
        let v = Validation {
            algorithms: vec![Self::ALGORITHM],
            iss: Some(issuer.to_string()),
            leeway: Self::LEEWAY,
            validate_exp: true,
            validate_nbf: true,

            ..Validation::default()
        };

        match decode_token::<Self>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &v,
        ) {
            Ok(v) => Ok(v.claims),
            Err(e) => Err(e),
        }
    }

    fn encode(
        data: TokenData,
        issuer: &str,
        key_id: &str,
        secret: &str,
    ) -> String {
        // TODO: aud
        let c = Self {
            sub: data.value,
            iat: data.granted_at as usize,
            iss: issuer.to_string(),
            exp: data.expires_at as usize,
            nbf: data.granted_at as usize,
        };

        let h = Header {
            alg: Self::ALGORITHM,
            kid: Some(key_id.to_string()),
            ..Default::default()
        };
        encode_data(&h, &c, &EncodingKey::from_secret(secret.as_bytes()))
            .unwrap()
    }

    fn get_subject(&self) -> String {
        self.sub.to_string()
    }

    fn get_issued_at(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(self.iat as i64, 0)
    }

    fn get_expiration_time(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(self.exp as i64, 0)
    }
}

/// AuthenticationClaims
///
/// This claims is used for user's signin action and request with an access
/// token.
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthenticationClaims {
    pub sub: String,
    pub iat: usize,
    pub iss: String,
    pub exp: usize,
    pub nbf: usize,
}

impl Claims for AuthenticationClaims {
    const ALGORITHM: Algorithm = Algorithm::HS256;
    const LEEWAY: u64 = 36; // seconds

    fn decode(
        token: &str,
        issuer: &str,
        secret: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error> {
        // self check
        let header = decode_header(token)?;
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

        match decode_token::<Self>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &v,
        ) {
            Ok(v) => Ok(v.claims),
            Err(e) => Err(e),
        }
    }

    fn encode(
        data: TokenData,
        issuer: &str,
        key_id: &str,
        secret: &str,
    ) -> String {
        // TODO: aud
        let c = Self {
            sub: data.value,
            iat: data.granted_at as usize,
            iss: issuer.to_string(),
            exp: data.expires_at as usize,
            nbf: data.granted_at as usize,
        };

        let h = Header {
            alg: Self::ALGORITHM,
            kid: Some(key_id.to_string()),
            ..Default::default()
        };
        encode_data(&h, &c, &EncodingKey::from_secret(secret.as_bytes()))
            .unwrap()
    }

    fn get_subject(&self) -> String {
        self.sub.to_string()
    }

    fn get_issued_at(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(self.iat as i64, 0)
    }

    fn get_expiration_time(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp(self.exp as i64, 0)
    }
}

pub type BrowserCookieTokenClaims = AuthenticationClaims;
pub type PersonalAccessTokenClaims = AuthenticationClaims;

#[cfg(test)]
mod test {
    use super::*;

    use base64::decode;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use rstest::rstest;

    use crate::model::test::CONFIG;

    #[test]
    fn token_format() {
        let now = Utc::now();
        let ts = now.timestamp();

        let t = TokenData {
            value: "dummy".to_string(),
            granted_at: ts,
            expires_at: ts,
        };

        assert_eq!(format!("{}", t), "dummy");
    }

    #[test]
    fn verification_claims_encode() {
        let now = Utc.ymd(2019, 6, 11).and_hms(23, 19, 32);
        let data = TokenData {
            value: "dummy".to_string(),
            granted_at: now.timestamp(),
            expires_at: (now + Duration::hours(24)).timestamp(),
        };

        let token = VerificationClaims::encode(
            data.clone(),
            "issuer",
            "key_id",
            "secret",
        );

        assert_eq!(
            token,
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

        let s: Vec<&str> = token.split('.').collect();
        let body = &decode(s[1]).unwrap()[..]; // base64
        let json = String::from_utf8_lossy(body).to_string();

        let claims: VerificationClaims =
            serde_json::from_str(&json).ok().unwrap();
        assert_eq!(claims.sub, data.value);
        assert_eq!(claims.iss, "issuer");
        assert_eq!(claims.iat, 1_560_295_172);

        let one_day = 86400; // 60 * 60 * 24
        assert_eq!(claims.exp, claims.iat + one_day); // 1560381572

        assert_eq!(claims.nbf, 1_560_295_172);
    }

    #[rstest(
        value, issuer, secret, granted_at,
        case( // expires
            "dummy",
            &CONFIG.verification_token_issuer,
            &CONFIG.verification_token_secret,
            Utc.ymd(2001, 1, 1).and_hms(10, 0, 0),
        ),
        case( // not before (future)
            "dummy",
            &CONFIG.verification_token_issuer,
            &CONFIG.verification_token_secret,
            Utc::now() + Duration::hours(3),
        ),
        case( // wrong issuer
            "dummy",
            "unknown",
            &CONFIG.verification_token_secret,
            Utc::now(),
        ),
        case( // invalid secret
            "dummy",
            &CONFIG.verification_token_issuer,
            "invalid",
            Utc::now(),
        )
        ::trace
    )]
    #[test]
    fn verification_claims_decode_failure(
        value: &'static str,
        issuer: &'static str,
        secret: &'static str,
        granted_at: DateTime<Utc>,
    ) {
        let data = TokenData {
            value: value.to_string(),
            granted_at: granted_at.timestamp(),
            expires_at: (granted_at + Duration::hours(24)).timestamp(),
        };
        let token = VerificationClaims::encode(
            data,
            &CONFIG.verification_token_issuer,
            &CONFIG.verification_token_key_id,
            &CONFIG.verification_token_secret,
        );
        assert!(VerificationClaims::decode(&token, issuer, secret).is_err());
    }

    #[rstest(
        value, issuer, secret, granted_at,
        case( // within limit
            "dummy",
            &CONFIG.verification_token_issuer,
            &CONFIG.verification_token_secret,
            Utc::now() - Duration::hours(3), // will be created at compile
        ),
        case( // now
            "dummy",
            &CONFIG.verification_token_issuer,
            &CONFIG.verification_token_secret,
            Utc::now(),
        ),
        ::trace
    )]
    #[test]
    fn verification_claims_decode_success(
        value: &'static str,
        issuer: &'static str,
        secret: &'static str,
        granted_at: DateTime<Utc>,
    ) {
        dbg!(&granted_at);
        let data = TokenData {
            value: value.to_string(),
            granted_at: granted_at.timestamp(),
            expires_at: (granted_at + Duration::hours(24)).timestamp(),
        };

        let token = VerificationClaims::encode(
            data.clone(),
            &CONFIG.verification_token_issuer,
            &CONFIG.verification_token_key_id,
            &CONFIG.verification_token_secret,
        );

        let claims = VerificationClaims::decode(&token, issuer, secret)
            .ok()
            .unwrap();

        assert_eq!(claims.sub, data.value);
        assert_eq!(claims.iss, CONFIG.verification_token_issuer);
        assert_eq!(claims.iat, data.granted_at as usize);

        let one_day = 86400; // 60 * 60 * 24
        assert_eq!(claims.exp, claims.iat + one_day); // 1560381572

        assert_eq!(claims.nbf, data.granted_at as usize);
    }
}
