use std::collections::HashMap;
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

pub(crate) fn counts(tokens: Vec<String>) -> Vec<(String, f32)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for token in tokens {
        *counts.entry(token.as_str().to_string()).or_default() += 1;
    }
    counts.into_iter().map(|(k, v)| (k, v as f32)).collect()
}
