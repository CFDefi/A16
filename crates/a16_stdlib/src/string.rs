//! String Standard Library
//!
//! String manipulation utilities.

/// Check if string starts with prefix
pub fn starts_with(s: &str, prefix: &str) -> bool { s.starts_with(prefix) }

/// Check if string ends with suffix
pub fn ends_with(s: &str, suffix: &str) -> bool { s.ends_with(suffix) }

/// Convert to uppercase
pub fn to_upper(s: &str) -> String { s.to_uppercase() }

/// Convert to lowercase
pub fn to_lower(s: &str) -> String { s.to_lowercase() }

/// Capitalize first letter
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let upper: String = first.to_uppercase().collect();
            upper + chars.as_str()
        }
    }
}

/// Title case (capitalize each word)
pub fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| capitalize(word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Trim whitespace from both ends
pub fn trim(s: &str) -> String { s.trim().to_string() }

/// Trim whitespace from start
pub fn trim_start(s: &str) -> String { s.trim_start().to_string() }

/// Trim whitespace from end
pub fn trim_end(s: &str) -> String { s.trim_end().to_string() }

/// Pad start with character to reach target length
pub fn pad_start(s: &str, target_len: usize, pad_char: char) -> String {
    if s.len() >= target_len {
        return s.to_string();
    }
    let padding: String = std::iter::repeat(pad_char).take(target_len - s.len()).collect();
    format!("{}{}", padding, s)
}

/// Pad end with character to reach target length
pub fn pad_end(s: &str, target_len: usize, pad_char: char) -> String {
    if s.len() >= target_len {
        return s.to_string();
    }
    let padding: String = std::iter::repeat(pad_char).take(target_len - s.len()).collect();
    format!("{}{}", s, padding)
}

/// Repeat string n times
pub fn repeat(s: &str, n: usize) -> String { s.repeat(n) }

/// Replace all occurrences
pub fn replace(s: &str, from: &str, to: &str) -> String { s.replace(from, to) }

/// Replace first occurrence
pub fn replace_first(s: &str, from: &str, to: &str) -> String { s.replacen(from, to, 1) }

/// Split string by delimiter
pub fn split(s: &str, delimiter: &str) -> Vec<String> {
    s.split(delimiter).map(|p| p.to_string()).collect()
}

/// Split into lines
pub fn lines(s: &str) -> Vec<String> {
    s.lines().map(|l| l.to_string()).collect()
}

/// Join strings with separator
pub fn join(parts: &[String], separator: &str) -> String {
    parts.join(separator)
}

/// Check if string contains substring
pub fn contains(s: &str, sub: &str) -> bool { s.contains(sub) }

/// Find index of substring (-1 if not found)
pub fn index_of(s: &str, sub: &str) -> i64 {
    s.find(sub).map(|i| i as i64).unwrap_or(-1)
}

/// Count occurrences of substring
pub fn count(s: &str, sub: &str) -> usize { s.matches(sub).count() }

/// Reverse a string
pub fn reverse(s: &str) -> String { s.chars().rev().collect() }

/// Check if string is empty
pub fn is_empty(s: &str) -> bool { s.is_empty() }

/// Check if string is all whitespace
pub fn is_blank(s: &str) -> bool { s.trim().is_empty() }

/// Check if string is all digits
pub fn is_digit(s: &str) -> bool { !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) }

/// Check if string is all alphabetic
pub fn is_alpha(s: &str) -> bool { !s.is_empty() && s.chars().all(|c| c.is_alphabetic()) }

/// Check if string is alphanumeric
pub fn is_alnum(s: &str) -> bool { !s.is_empty() && s.chars().all(|c| c.is_alphanumeric()) }

/// Substring
pub fn substring(s: &str, start: usize, end: usize) -> String {
    s.chars().skip(start).take(end.saturating_sub(start)).collect()
}

/// Character at index
pub fn char_at(s: &str, index: usize) -> Option<char> {
    s.chars().nth(index)
}

/// String length in characters (not bytes)
pub fn char_count(s: &str) -> usize { s.chars().count() }
