use rand::prelude::*;
use rocket::http::{Cookie, SameSite};

// Creates random hash based on source characters
pub fn generate_random_hash(source: &[u8], length: i32) -> String {
    if length < 1 {
        return "".to_string();
    }

    let mut rng = thread_rng();
    let source_length = source.len();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0, source_length);
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
pub fn make_cookie<'a>(sign: String) -> Cookie<'a> {
    // TODO:
    // consider about extension (re-set it again?)
    let mut signature = Cookie::new("sign", sign);
    signature.set_domain("127.0.0.1");
    signature.set_same_site(SameSite::Strict);
    signature.set_secure(false); // FIXME
    signature.set_http_only(true);
    signature
}

#[cfg(test)]
mod test {
    use super::*;

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
}
