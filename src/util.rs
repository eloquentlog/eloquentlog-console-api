use rand::prelude::*;

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

#[cfg(test)]
mod util_test {
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
