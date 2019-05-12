pub mod message;
pub mod user;

use accord::{Invalid, ValidatorResult};
use accord::validators::{alphanumeric, max as original_max};

type SV = Box<Fn(&String) -> ValidatorResult>;

const CHARS_LOWER: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const CHARS_UPPER: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const DIGITS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

fn alphanumeric_underscore_if_present(
) -> Box<Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        match &s {
            Some(v) => alphanumeric()(&v.replace("_", "")),
            None => Ok(()),
        }
    })
}

fn contain_any(accepted: &'static [char], text: &'static str) -> SV {
    Box::new(move |s: &String| {
        for c in s.chars() {
            if accepted.contains(&c) {
                return Ok(());
            }
        }
        Err(Invalid {
            msg: "Must contain %1".to_string(),
            args: vec![text.to_string()],
            human_readable: format!("Must contain '{}'", text),
        })
    })
}

fn not_contain_only_digits_or_underscore_if_present(
) -> Box<Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        match &s {
            Some(v) => {
                for c in v.chars() {
                    if !DIGITS.contains(&c) && c != '_' {
                        return Ok(());
                    }
                }
                Err(Invalid {
                    msg: "Must not contain only digits or underscore"
                        .to_string(),
                    args: vec![],
                    human_readable: "Must not contain only digits or \
                                     underscore"
                        .to_string(),
                })
            },
            None => Ok(()),
        }
    })
}

fn not_contain_if_given(needle: Option<String>) -> SV {
    let n = needle.unwrap_or_default();
    Box::new(move |s: &String| {
        // not contain and not included in
        if !n.is_empty() && (s.contains(&n) || n.contains(s)) {
            let s = s.to_string();
            let v = if n > s { &n } else { &s };
            return Err(Invalid {
                msg: "Must not contain %1.".to_string(),
                args: vec![v.to_string()],
                human_readable: format!("Must not contain '{}'", v),
            });
        }
        Ok(())
    })
}

fn not_overlap_with(field: &'static str) -> Box<Fn(Option<String>) -> SV> {
    Box::new(move |needle: Option<String>| {
        let f = field.to_string();
        Box::new(move |s: &String| {
            let n = needle.clone();
            if not_contain_if_given(n)(s).is_err() {
                return Err(Invalid {
                    msg: "Must not overlap with %1.".to_string(),
                    args: vec![f.to_string()],
                    human_readable: format!("Must not overlap with {}", f),
                });
            }
            Ok(())
        })
    })
}

fn not_start_with_if_present(
    needle: &'static str,
) -> Box<Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        match &s {
            Some(v) if !v.is_empty() && v.replacen(needle, "", 1) == v[1..] => {
                Err(Invalid {
                    msg: "Must not start with '%1'".to_string(),
                    args: vec![needle.to_string()],
                    human_readable: format!("Must not start with '{}'", needle),
                })
            },
            _ => Ok(()),
        }
    })
}

fn not_start_with_digits_if_present(
) -> Box<Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        match &s {
            Some(v) if !v.is_empty() => {
                if !DIGITS.contains(&v.chars().next().unwrap()) {
                    return Ok(());
                }
                Err(Invalid {
                    msg: "Must not start with digits".to_string(),
                    args: vec![],
                    human_readable: "Must not start with digits".to_string(),
                })
            },
            _ => Ok(()),
        }
    })
}

fn required() -> Box<Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        if s.is_some() {
            return Ok(());
        }
        Err(Invalid {
            msg: "Must exist".to_string(),
            args: vec![],
            human_readable: "Must exist".to_string(),
        })
    })
}

fn max_if_present(max: usize) -> Box<Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        match &s {
            Some(v) => original_max(max)(&v),
            None => Ok(()),
        }
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_alphanumeric_underscore_if_present() {
        let tests: [(&'static str, bool); 5] = [
            ("$", false),
            ("(text)", false),
            ("0-o", false),
            ("_123", true),
            ("ab_123", true),
        ];

        for (i, (s, expected)) in tests.iter().enumerate() {
            assert_eq!(
                *expected,
                alphanumeric_underscore_if_present()(&Some(s.to_string()))
                    .is_ok(),
                "#{} value: {}",
                i,
                s
            );
        }
    }

    #[test]
    fn test_not_contain_only_digits_or_underscore_if_present() {
        let tests: [(&'static str, bool); 5] = [
            ("123456789", false),
            ("123_456", false),
            ("0___", false),
            ("u123", true),
            ("123four", true),
        ];

        for (i, (s, expected)) in tests.iter().enumerate() {
            assert_eq!(
                *expected,
                not_contain_only_digits_or_underscore_if_present()(&Some(
                    s.to_string()
                ))
                .is_ok(),
                "#{} value: {}",
                i,
                s
            );
        }
    }

    #[test]
    fn test_not_contain_if_given() {
        let tests: [(&'static str, &'static str, bool); 5] = [
            ("@", "@23456789", false),
            ("_", "123__45", false),
            ("a", "0a___", false),
            ("_", "u123", true),
            ("4", "123four", true),
        ];

        for (i, (given, s, expected)) in tests.iter().enumerate() {
            assert_eq!(
                *expected,
                not_contain_if_given(Some(given.to_string()))(&s.to_string())
                    .is_ok(),
                "#{} given: {} value: {}",
                i,
                given,
                s
            );
        }
    }

    #[test]
    fn test_not_overlap_with() {
        let tests: [(&'static str, &'static str, bool); 5] = [
            ("abcdef", "cdef", false),
            ("abcdef", "abcdefghijk", false),
            ("aaaa", "aaaa___", false),
            ("!!!", "@123_45", true),
            ("4", "123four", true),
        ];

        let field_name = "test_field";

        for (i, (needle, s, expected)) in tests.iter().enumerate() {
            assert_eq!(
                *expected,
                not_overlap_with(field_name)(Some(needle.to_string()))(
                    &s.to_string()
                )
                .is_ok(),
                "#{} field_name: {}, needle: {} value: {}",
                i,
                field_name,
                needle,
                s
            );
        }
    }

    #[test]
    fn test_not_start_with_if_present() {
        let tests: [(&'static str, &'static str, bool); 5] = [
            ("@", "@23456789", false),
            ("_", "__12345", false),
            ("0", "0___", false),
            ("_", "u123", true),
            ("4", "123four", true),
        ];

        for (i, (needle, s, expected)) in tests.iter().enumerate() {
            assert_eq!(
                *expected,
                not_start_with_if_present(needle)(&Some(s.to_string())).is_ok(),
                "#{} needle: {} value: {}",
                i,
                needle,
                s
            );
        }
    }

    #[test]
    fn test_not_start_with_digits_if_present() {
        let tests: [(&'static str, bool); 5] = [
            ("0123456789", false),
            ("12345", false),
            ("0", false),
            ("u123", true),
            ("_123four", true),
        ];

        for (i, (s, expected)) in tests.iter().enumerate() {
            assert_eq!(
                *expected,
                not_start_with_digits_if_present()(&Some(s.to_string()))
                    .is_ok(),
                "#{} value: {}",
                i,
                s
            );
        }
    }

    #[test]
    fn test_required() {
        assert!(required()(&None).is_err());
        assert!(required()(&Some("".to_string())).is_ok());
    }

    #[test]
    fn test_max_if_present() {
        assert!(max_if_present(3)(&Some("1234".to_string())).is_err());
        assert!(max_if_present(3)(&Some("123".to_string())).is_ok());
        assert!(max_if_present(0)(&Some("".to_string())).is_ok());
        assert!(max_if_present(3)(&None).is_ok());
        assert!(max_if_present(0)(&None).is_ok());
    }
}
