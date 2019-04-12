pub mod message;
pub mod user;

use accord::{Invalid, ValidatorResult};
use accord::validators::max as original_max;

fn not_contain_if_given(
    needle: Option<String>,
) -> Box<Fn(&String) -> ValidatorResult> {
    let n = needle.unwrap_or_default();
    Box::new(move |s: &String| {
        if !n.is_empty() && (s.contains(&n) || n.contains(s)) {
            let v = if &n > s { &n } else { s };
            return Err(Invalid {
                msg: "Must not contain %1.".to_string(),
                args: vec![v.to_string()],
                human_readable: format!("Must not contain '{}'", v),
            });
        }
        Ok(())
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
