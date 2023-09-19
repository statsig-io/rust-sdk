use std::cmp::max;
use std::mem::size_of;

use chrono::Duration;
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::statsig::internal::helpers::UsizeExt;

pub fn compute_user_hash(value: String) -> Option<usize> {
    let mut sha256 = Sha256::new();
    sha256.update(value.as_str().as_bytes());
    let result = sha256.finalize();
    match result.split_at(size_of::<usize>()).0.try_into() {
        Ok(bytes) => Some(usize::from_be_bytes(bytes)),
        _ => None,
    }
}

pub fn compare_numbers(left: &Value, right: &Value, op: &str) -> Option<bool> {
    let left_num = value_to_f64(left)?;
    let right_num = value_to_f64(right)?;
    match op {
        "gt" => Some(left_num > right_num),
        "gte" => Some(left_num >= right_num),
        "lt" => Some(left_num < right_num),
        "lte" => Some(left_num <= right_num),
        _ => None,
    }
}

pub fn compare_versions(left: &Value, right: &Value, op: &str) -> Option<bool> {
    let mut left_str = value_to_string(left)?;
    let mut right_str = value_to_string(right)?;

    if let Some(index) = left_str.find("-") {
        left_str = left_str[0..index].to_string();
    }

    if let Some(index) = right_str.find("-") {
        right_str = right_str[0..index].to_string();
    }

    fn comparison(left_str: &String, right_str: &String) -> Option<i32> {
        let left_parts: Vec<&str> = left_str.split(".").collect();
        let right_parts: Vec<&str> = right_str.split(".").collect();

        let mut i = 0;
        while i < max(left_parts.len(), right_parts.len()) {
            let (mut left_count, mut right_count) = (0, 0);

            if i < left_parts.len() {
                left_count = left_parts[i].parse().ok()?;
            }

            if i < right_parts.len() {
                right_count = right_parts[i].parse().ok()?;
            }

            if left_count < right_count {
                return Some(-1);
            }

            if left_count > right_count {
                return Some(1);
            }

            i.post_inc();
        }
        Some(0)
    }

    let result = comparison(&left_str, &right_str)?;
    match op {
        "version_gt" => Some(result > 0),
        "version_gte" => Some(result >= 0),
        "version_lt" => Some(result < 0),
        "version_lte" => Some(result <= 0),
        "version_eq" => Some(result == 0),
        "version_neq" => Some(result != 0),
        _ => None,
    }
}

pub fn compare_strings_in_array(value: &Value, array: &Value, op: &str, ignore_case: bool) -> bool {
    let comparison = || {
        let value_str = value_to_string(value)?;
        Some(array.as_array()?.iter().any(|current| {
            let curr_str = match value_to_string(current) {
                Some(s) => s,
                _ => return false,
            };
            let left = if ignore_case {
                value_str.to_lowercase()
            } else {
                value_str.clone()
            };
            let right = if ignore_case {
                curr_str.to_lowercase()
            } else {
                curr_str.clone()
            };

            match op {
                "any" | "none" | "any_case_sensitive" | "none_case_sensitive" => left.eq(&right),
                "str_starts_with_any" => left.starts_with(&right),
                "str_ends_with_any" => left.ends_with(&right),
                "str_contains_any" | "str_contains_none" => left.contains(&right),
                _ => false,
            }
        }))
    };

    let res = comparison().unwrap_or(false);

    if op == "none" || op == "none_case_sensitive" || op == "str_contains_none" {
        return !res;
    }
    res
}

pub fn compare_str_with_regex(value: &Value, regex_value: &Value) -> bool {
    let comparison = || {
        let value_str = value_to_string(value)?;
        let regex_str = value_to_string(regex_value)?;
        let regex = Regex::new(&regex_str).ok()?;
        Some(regex.is_match(&value_str))
    };

    comparison().unwrap_or(false)
}

pub fn compare_time(left: &Value, right: &Value, op: &str) -> Option<bool> {
    let left_num = value_to_i64(left)?;
    let right_num = value_to_i64(right)?;

    match op {
        "before" => Some(left_num < right_num),
        "after" => Some(left_num > right_num),
        "on" => Some(
            Duration::milliseconds(left_num).num_days()
                == Duration::milliseconds(right_num).num_days(),
        ),
        _ => None,
    }
}

pub fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

pub fn value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

pub fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        _ => Some(format!("{}", value)),
    }
}
