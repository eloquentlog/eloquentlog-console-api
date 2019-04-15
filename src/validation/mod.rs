pub mod message;
pub mod user;

use accord::{Invalid, ValidatorResult};
use accord::validators::{
alphanumeric,
max as original_max,
};

type SV = Box<Fn(&String) -> ValidatorResult>;

const DIGITS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub fn alphanumeric_underscore_if_present(
) -> Box<Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        match &s {
            Some(v) => alphanumeric()(&v.replace("_", "")),
            None => Ok(()),
        }
    })
}

pub fn not_contain_only_digits_or_underscore_if_present(
) -> Box<Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        match &s {
            Some(v) => {
                for c in v.replace("_", "").chars() {
                    if !DIGITS.contains(&c) {
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

pub fn not_overlap_with(field: &'static str) -> Box<Fn(Option<String>) -> SV> {
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

pub fn required() -> Box<Fn(&Option<String>) -> ValidatorResult> {
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

pub fn max_if_present(
    max: usize,
) -> Box<Fn(&Option<String>) -> ValidatorResult> {
    Box::new(move |s: &Option<String>| {
        match &s {
            Some(v) => original_max(max)(&v),
            None => Ok(()),
        }
    })
}
