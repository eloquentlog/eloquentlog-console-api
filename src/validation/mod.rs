pub mod message;
pub mod password_reset;
pub mod password_reset_request;
pub mod user;

use accord::{Invalid, ValidatorResult};
use accord::validators::{alphanumeric, max as original_max};

type SV = Box<dyn Fn(&String) -> ValidatorResult>;

const CHARS_LOWER: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const CHARS_UPPER: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const DIGITS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub messages: Vec<String>,
}

fn contain_only_alphanumeric_or_underscore(
) -> Box<dyn Fn(&String) -> ValidatorResult> {
    Box::new(move |s: &String| alphanumeric()(&s.replace("_", "")))
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

fn not_contain_only_digits_or_underscore(
) -> Box<dyn Fn(&String) -> ValidatorResult> {
    Box::new(move |s: &String| {
        for c in s.chars() {
            if !DIGITS.contains(&c) && c != '_' {
                return Ok(());
            }
        }
        Err(Invalid {
            msg: "Must not contain only digits or underscore".to_string(),
            args: vec![],
            human_readable: "Must not contain only digits or underscore"
                .to_string(),
        })
    })
}

// check if the needle is given (present)
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

fn not_overlap_with(field: &'static str) -> Box<dyn Fn(String) -> SV> {
    Box::new(move |needle: String| {
        let f = field.to_string();
        Box::new(move |s: &String| {
            let n = needle.clone();
            if not_contain_if_given(Some(n))(s).is_err() {
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

fn not_start_with(
    needle: &'static str,
) -> Box<dyn Fn(&String) -> ValidatorResult> {
    Box::new(move |s: &String| {
        if !s.is_empty() && s.replacen(needle, "", 1) == s[1..] {
            return Err(Invalid {
                msg: "Must not start with '%1'".to_string(),
                args: vec![needle.to_string()],
                human_readable: format!("Must not start with '{}'", needle),
            });
        }
        Ok(())
    })
}

fn not_start_with_digits() -> Box<dyn Fn(&String) -> ValidatorResult> {
    Box::new(move |s: &String| {
        if !s.is_empty() && DIGITS.contains(&s.chars().next().unwrap()) {
            return Err(Invalid {
                msg: "Must not start with digits".to_string(),
                args: vec![],
                human_readable: "Must not start with digits".to_string(),
            });
        }
        Ok(())
    })
}

fn required() -> Box<dyn Fn(&Option<String>) -> ValidatorResult> {
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

fn max_if_present(
    max: usize,
) -> Box<dyn Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        match &s {
            Some(v) => original_max(max)(&v),
            None => Ok(()),
        }
    })
}

#[rustfmt::skip::attributes(rstest)]
#[cfg(test)]
mod test {
    use super::*;

    use rstest::rstest;

    #[rstest(
        raw_s, expected,
        case("$", false),
        case("(text)", false),
        case("0-o", false),
        case("_123", true),
        case("ab_123", true),
        ::trace
    )]
    #[test]
    fn test_contain_only_alphanumeric_or_underscore(
        raw_s: &'static str,
        expected: bool,
    ) {
        let f = contain_only_alphanumeric_or_underscore();
        let s = &raw_s.to_string();

        assert_eq!(expected, f(s).is_ok());
    }

    #[rstest(raw_s, expected,
        case("123456789", false),
        case("123_456", false),
        case("0___", false),
        case("u123", true),
        case("123four", true),
        ::trace
    )]
    #[test]
    fn test_not_contain_only_digits_or_underscore(
        raw_s: &'static str,
        expected: bool,
    ) {
        let f = not_contain_only_digits_or_underscore();
        let s = &raw_s.to_string();

        assert_eq!(expected, f(s).is_ok());
    }

    #[rstest(
        needle, raw_s, expected,
        case("@", "@23456789", false),
        case("_", "123__45", false),
        case("a", "0a___", false),
        case("_", "u123", true),
        case("4", "123four", true),
        ::trace
    )]
    #[test]
    fn test_not_contain_if_given(
        needle: &'static str,
        raw_s: &'static str,
        expected: bool,
    ) {
        let f = not_contain_if_given(Some((needle).to_string()));
        let s = &raw_s.to_string();

        assert_eq!(expected, f(s).is_ok());
    }

    #[rstest(
        needle, raw_s, expected,
        case("abcdef", "cdef", false),
        case("abcdef", "abcdefghijk", false),
        case("aaaa", "aaaa___", false),
        case("!!!", "@123_45", true),
        case("4", "123four", true),
        ::trace
    )]
    #[test]
    fn test_not_overlap_with(
        needle: &'static str,
        raw_s: &'static str,
        expected: bool,
    ) {
        let field_name = "test_field";

        let f = not_overlap_with(field_name)(needle.to_string());
        let s = &raw_s.to_string();

        assert_eq!(expected, f(s).is_ok());
    }

    #[rstest(
        needle, raw_s, expected,
        case("@", "@23456789", false),
        case("_", "__12345", false),
        case("0", "0___", false),
        case("_", "u123", true),
        case("4", "123four", true),
        ::trace
    )]
    #[test]
    fn test_not_start_with(
        needle: &'static str,
        raw_s: &'static str,
        expected: bool,
    ) {
        let f = not_start_with(needle);
        let s = &raw_s.to_string();

        assert_eq!(expected, f(s).is_ok());
    }

    #[rstest(
        raw_s, expected,
        case("0123456789", false),
        case("12345", false),
        case("0", false),
        case("u123", true),
        case("_123four", true),
        ::trace
    )]
    #[test]
    fn test_not_start_with_digits(raw_s: &'static str, expected: bool) {
        let f = not_start_with_digits();
        let s = &raw_s.to_string();

        assert_eq!(expected, f(s).is_ok());
    }

    #[rstest(
        raw_s, expected,
        case(None, false),
        case(Some("".to_string()), true),
        ::trace
    )]
    #[test]
    fn test_required(raw_s: Option<String>, expected: bool) {
        let f = required();
        let s = &raw_s;

        assert_eq!(expected, f(s).is_ok());
    }

    #[rstest(
        max, raw_s, expected,
        case(3, Some("1234".to_string()), false),
        case(3, Some("123".to_string()), true),
        case(0, Some("".to_string()), true),
        case(3, None, true),
        case(0, None, true),
        ::trace
    )]
    #[test]
    fn test_max_if_present(max: usize, raw_s: Option<String>, expected: bool) {
        let f = max_if_present(max);
        let s = &raw_s;

        assert_eq!(expected, f(s).is_ok());
    }
}
