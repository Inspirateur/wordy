use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
const OPENING_PUNCT: &[char] = &['(', '[', '{', '\'', '"', '*', '`'];
const CLOSING_PUNCT: &[char] = &[':', '.', '?', '!', '`', ';', ',', ')', ']', '}', '\'', '"', '*', '`'];

lazy_static! {
    static ref RE_TOKEN: Regex = Regex::new(r"\S+").unwrap();
}

fn is_capitalized(token: &str) -> bool {
    !token.chars().skip(1).all(|c| c.is_lowercase())
}

fn smart_lower(token: &str) -> String {
    if is_capitalized(token) {
        token.to_lowercase()
    } else {
        token.to_string()
    }
}

fn trim(token: &str) -> &str {
    token.trim_start_matches(OPENING_PUNCT)
        .trim_end_matches(CLOSING_PUNCT)
}

pub fn tokenize(text: String) -> Vec<String> {
    RE_TOKEN.find_iter(&text)
        .map(|token| smart_lower(trim(token.as_str())))
        .collect_vec()
}
