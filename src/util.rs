use rand::prelude::*;
use rocket::http::{Cookie, SameSite};
use rocket::Request;

use crate::config::Config;

// Creates random hash based on source characters
pub fn generate_random_hash(source: &[u8], length: i32) -> String {
    if length < 1 {
        return "".to_string();
    }

    let mut rng = thread_rng();
    let source_length = source.len();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..source_length);
            char::from(unsafe { *source.get_unchecked(idx) })
        })
        .collect()
}

pub fn split_token(token: String) -> Option<(String, String)> {
    let parts: Vec<&str> = token.split('.').collect();
    // unexpected
    if parts.len() != 3 {
        return None;
    }

    // NOTE:
    // JS should handle this into permanent cookies with expires.
    // The value should be composed from `header.payload`.
    // TODO:
    // consider about implementation "Are you there?" modal
    let payload = parts[0..2].join(".");

    Some((payload, parts[2].to_string()))
}

// Make a cookie for signature (sign).
//
// This is session cookie (no expires and max-age)
pub fn make_cookie<'a>(sign: String, config: &Config) -> Cookie<'a> {
    let mut sig = Cookie::new("sign", sign);
    sig.set_domain(config.cookie_domain.to_owned());
    sig.set_path("/");
    sig.set_same_site(SameSite::Strict);
    sig.set_secure(config.cookie_secure);
    sig.set_http_only(true);
    sig
}

/// Extract session key with a prefix from path
///
/// The URI path should look like:
/// * /_/password/reset/<...>
/// * /_/activate/<...>
pub fn extract_session_key(req: &Request<'_>) -> String {
    // NOTE: The part of `/_/` (empty segment) will be ignored in routed path
    // within Segments. See below:
    // https://api.rocket.rs/v0.4/rocket/http/uri/struct.Segments.html
    let s0 = req
        .raw_segment_str(0)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "".to_string());
    let (idx, pfx) = if s0 == "password" {
        (2, "pr")
    } else if s0 == "activate" {
        (1, "ua")
    } else {
        return "".to_string();
    };
    let sn = req
        .raw_segment_str(idx)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "".to_string());
    if sn.is_empty() {
        return "".to_string();
    }
    format!("{}-{}", pfx, sn)
}

#[cfg(test)]
mod test {
    use super::*;

    use rocket::http::Method;
    use rocket::http::uri::Origin;
    use rocket::local::Client;

    #[test]
    fn test_generate_random_hash_length() {
        let s = b".";
        assert_eq!(generate_random_hash(s, -1), "".to_string());
        assert_eq!(generate_random_hash(s, 0), "".to_string());

        assert_eq!(generate_random_hash(s, 1), ".".to_string());
        assert_eq!(generate_random_hash(s, 3), "...".to_string());
        assert_eq!(generate_random_hash(s, 6), "......".to_string());
        assert_eq!(generate_random_hash(s, 9), ".........".to_string());
    }

    #[test]
    fn test_generate_random_hash_source() {
        let s = b"abcdefghijklmnopqrstuvwxyz";
        let t = String::from_utf8_lossy(s);
        let value = generate_random_hash(s, 128);

        for v in value.chars() {
            assert!(t.contains(v));
        }
    }

    #[test]
    fn test_extract_session_key() {
        let client = Client::new(rocket::ignite()).expect("valid rocket");

        let local = client.req(Method::Get, "/");
        let mut req = local.inner().clone();

        let uri = Origin::parse("/").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "");

        let uri = Origin::parse("/unkonwn").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "");

        let uri = Origin::parse("/password/reset").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "");

        let uri = Origin::parse("/password/reset/").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "");

        let uri = Origin::parse("/password/reset/123").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "pr-123");

        let uri = Origin::parse("/password/reset/123/").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "pr-123");

        let uri = Origin::parse("/password/reset/123/456").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "pr-123");

        let uri = Origin::parse("/activate").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "");

        let uri = Origin::parse("/activate/").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "");

        let uri = Origin::parse("/activate/456").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "ua-456");

        let uri = Origin::parse("/activate/456/").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "ua-456");

        let uri = Origin::parse("/activate/456/789").unwrap();
        req.set_uri(uri);
        assert_eq!(extract_session_key(&req), "ua-456");
    }
}
