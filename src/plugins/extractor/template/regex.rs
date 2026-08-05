use super::css;
use regex::Regex;

pub(crate) fn extract(body: &str, pattern: &str, base: &str) -> Option<String> {
    Regex::new(pattern).ok().and_then(|regex| {
        regex.captures(body).and_then(|capture| {
            capture
                .get(1)
                .or_else(|| capture.get(0))
                .map(|value| css::resolve(base, value.as_str()))
        })
    })
}
