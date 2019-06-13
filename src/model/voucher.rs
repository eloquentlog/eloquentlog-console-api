//! Claims and VoucherData
use std::fmt;

use chrono::{Utc, DateTime, Duration};
use jsonwebtoken::{
    Algorithm, Header, Validation, decode as decode_token, decode_header,
    encode as encode_token,
};

pub struct VoucherData {
    pub value: String,
    // timestamp values
    pub expires_at: i64,
    pub granted_at: i64,
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
        now: DateTime<Utc>,
    ) -> VoucherData;

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

        // validate aud
        let v = Validation {
            algorithms: vec![Self::ALGORITHM],
            iss: Some(issuer.to_string()),
            leeway: Self::LEEWAY,
            validate_exp: true,
            validate_nbf: true,

            ..Validation::default()
        };

        match decode_token::<Self>(&value, secret.as_ref(), &v) {
            Ok(v) => Ok(v.claims),
            Err(e) => Err(e),
        }
    }

    fn encode(
        value: String,
        issuer: &str,
        key_id: &str,
        secret: &str,
        now: DateTime<Utc>,
    ) -> VoucherData
    {
        let granted_at = now.timestamp();
        let expires_at = (now + Duration::hours(24)).timestamp();

        // TODO: aud
        let c = Self {
            sub: value,
            iat: granted_at as usize,
            iss: issuer.to_string(),
            exp: expires_at as usize,
            nbf: granted_at as usize,
        };

        let mut h = Header::default();
        h.alg = Self::ALGORITHM;
        h.kid = Some(key_id.to_string());

        VoucherData {
            value: encode_token(&h, &c, secret.as_ref()).unwrap(),
            expires_at,
            granted_at,
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
    pub iat: usize,
    pub iss: String,
    pub exp: usize,
    pub nbf: usize,
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

        // validate aud
        let v = Validation {
            algorithms: vec![Self::ALGORITHM],
            iss: Some(issuer.to_string()),
            leeway: Self::LEEWAY,
            validate_exp: false,
            validate_nbf: true,

            ..Validation::default()
        };

        match decode_token::<Self>(&value, secret.as_ref(), &v) {
            Ok(v) => Ok(v.claims),
            Err(e) => Err(e),
        }
    }

    fn encode(
        value: String,
        issuer: &str,
        key_id: &str,
        secret: &str,
        now: DateTime<Utc>,
    ) -> VoucherData
    {
        let granted_at = now.timestamp();
        // TODO
        // set valid expires_at and impl review mechanism (check also
        // `validate_exp` for Validation struct)
        // let expires_at = (now + Duration::weeks(2)).timestamp();
        let expires_at = 0;

        // TODO: aud
        let c = Self {
            sub: value,
            iat: granted_at as usize,
            iss: issuer.to_string(),
            exp: expires_at as usize,
            nbf: granted_at as usize,
        };

        let mut h = Header::default();
        h.alg = Self::ALGORITHM;
        h.kid = Some(key_id.to_string());

        VoucherData {
            value: encode_token(&h, &c, secret.as_ref()).unwrap(),
            expires_at,
            granted_at,
        }
    }

    fn get_subject(&self) -> String {
        self.sub.to_string()
    }
}

#[cfg(test)]
mod voucher_test {
    use super::*;

    use chrono::TimeZone;
    use serde_json;

    extern crate base64;
    use self::base64::decode;

    #[test]
    fn test_voucher_data_format() {
        let now = Utc::now();
        let ts = now.timestamp();

        let u = VoucherData {
            value: "dummy".to_string(),
            expires_at: ts,
            granted_at: ts,
        };

        assert_eq!(format!("{}", u), "dummy");
    }

    #[test]
    fn test_activation_claims_encode() {
        let now = Utc.ymd(2019, 6, 11).and_hms(23, 19, 32);
        let value = "dummy".to_string();

        let voucher = ActivationClaims::encode(
            value.clone(),
            "issuer",
            "key_id",
            "secret",
            now,
        );

        let token = voucher.value;
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
        let token_body = &decode(s[1]).unwrap()[..]; // base64
        let json = String::from_utf8_lossy(token_body).to_string();

        let claims: ActivationClaims =
            serde_json::from_str(&json).ok().unwrap();
        assert_eq!(claims.sub, value);
        assert_eq!(claims.iss, "issuer");
        assert_eq!(claims.iat, 1_560_295_172);
        assert_eq!(claims.exp, claims.iat + 60 * 60 * 24); // +86400 (1560381572)
        assert_eq!(claims.nbf, 1_560_295_172);
    }

    #[test]
    fn test_activation_claims_decode() {
        let now = Utc::now();
        let value = "dummy".to_string();
        let voucher = ActivationClaims::encode(
            value.clone(),
            "issuer",
            "key_id",
            "secret",
            now,
        );

        let token = voucher.value;

        let claims = ActivationClaims::decode(&token, "issuer", "secret")
            .ok()
            .unwrap();

        let t = now.timestamp();
        assert_eq!(claims.sub, value);
        assert_eq!(claims.iss, "issuer");
        assert_eq!(claims.iat, t as usize);
        assert_eq!(claims.exp, claims.iat + 60 * 60 * 24); // +86400 (1560381572)
        assert_eq!(claims.nbf, t as usize);
    }
}
