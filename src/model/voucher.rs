//! Claims and VoucherData
use std::fmt;

use chrono::{Utc, Duration};
use jsonwebtoken::{Algorithm, Header, Validation, decode, decode_header, encode};

pub struct VoucherData {
    pub value: String,
    pub expires_at: i64,
}

impl fmt::Display for VoucherData {
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
        value: &str, // VoucherData's value
        issuer: &str,
        secret: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error>;

    fn encode(
        value: String, // subject
        issuer: &str,
        kei_id: &str,
        secret: &str,
    ) -> VoucherData;

    fn get_subject(&self) -> String;
}

/// ActivationClaims
#[derive(Debug, Deserialize, Serialize)]
pub struct ActivationClaims {
    pub sub: String,
    pub iss: String,
    pub exp: usize,
}

impl Claims for ActivationClaims {
    const ALGORITHM: Algorithm = Algorithm::HS512;
    const LEEWAY: i64 = 36; // seconds

    fn decode(
        value: &str,
        issuer: &str,
        secret: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error>
    {
        // self check
        let header = decode_header(&value).expect("Invalid token");
        if header.alg != Self::ALGORITHM {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }

        let v = Validation {
            algorithms: vec![Self::ALGORITHM],
            iss: Some(issuer.to_string()),
            leeway: Self::LEEWAY,
            validate_exp: true,

            ..Validation::default()
        };
        // TODO
        // validate subject
        match decode::<Self>(&value, secret.as_ref(), &v) {
            Ok(v) => Ok(v.claims),
            Err(e) => Err(e),
        }
    }

    fn encode(
        value: String,
        issuer: &str,
        key_id: &str,
        secret: &str,
    ) -> VoucherData
    {
        // TODO
        // iat (issue_at) and nbf (not before)
        let expires_at = (Utc::now() + Duration::hours(24)).timestamp();

        let c = Self {
            sub: value,
            iss: issuer.to_string(),
            exp: expires_at as usize,
        };

        let mut h = Header::default();
        h.alg = Self::ALGORITHM;
        h.kid = Some(key_id.to_string());

        VoucherData {
            value: encode(&h, &c, secret.as_ref()).unwrap(),
            expires_at,
        }
    }

    fn get_subject(&self) -> String {
        self.sub.to_string()
    }
}

/// AuthorizationClaims
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthorizationClaims {
    pub sub: String,
    pub iss: String,
    pub exp: usize,
}

impl Claims for AuthorizationClaims {
    const ALGORITHM: Algorithm = Algorithm::HS256;
    const LEEWAY: i64 = 36; // seconds

    fn decode(
        value: &str,
        issuer: &str,
        secret: &str,
    ) -> Result<Self, jsonwebtoken::errors::Error>
    {
        // self check
        let header = decode_header(&value).expect("Invalid token");
        if header.alg != Self::ALGORITHM {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ));
        }

        let v = Validation {
            algorithms: vec![Self::ALGORITHM],
            iss: Some(issuer.to_string()),
            leeway: Self::LEEWAY,
            validate_exp: false,

            ..Validation::default()
        };

        // TODO
        // validate subject
        match decode::<Self>(&value, secret.as_ref(), &v) {
            Ok(v) => Ok(v.claims),
            Err(e) => Err(e),
        }
    }

    fn encode(
        value: String,
        issuer: &str,
        key_id: &str,
        secret: &str,
    ) -> VoucherData
    {
        // TODO
        // iat (issue_at), (not before) and aud too
        let expires_at = (Utc::now() + Duration::hours(24)).timestamp();

        let c = Self {
            sub: value,
            iss: issuer.to_string(),
            exp: (Utc::now() + Duration::weeks(2)).timestamp() as usize,
        };

        let mut h = Header::default();
        h.alg = Self::ALGORITHM;
        h.kid = Some(key_id.to_string());

        VoucherData {
            value: encode(&h, &c, secret.as_ref()).unwrap(),
            expires_at,
        }
    }

    fn get_subject(&self) -> String {
        self.sub.to_string()
    }
}
