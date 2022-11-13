use std::mem::size_of;

use serde_json::Value;
use sha2::{Digest, Sha256};


pub fn compute_user_hash(value: String) -> Option<usize> {
    let mut sha256 = Sha256::new();
    sha256.update(value.as_str().as_bytes());
    let result = sha256.finalize();
    match result.split_at(size_of::<usize>()).0.try_into() {
        Ok(bytes) => Some(usize::from_be_bytes(bytes)),
        _ => None
    }
}

pub fn compare_numbers(left: &Value, right: &Value, op: &str) -> Option<bool> {
    let left_num = left.as_number()?;
    let right_num = right.as_number()?;
    match op {
        "gt" => Some(left_num > right_num),
        "gte" => Some(left_num >= right_num),
        "lt" => Some(left_num < right_num),
        "lte" => Some(left_num <= right_num),
        _ => None
    }
}

pub fn match_string_in_array(value: &Value, array: &Value, ignore_case: bool, op: &str) -> Option<bool> {
    if !value.is_string() {
        return None;
    }

    let value_str = value.to_string();
    let res = array.as_array()?.iter().any(|current| {
        if !current.is_string() {
            return false;
        }

        let curr_str = current.to_string();
        let left = if ignore_case { value_str.to_lowercase() } else { value_str.clone() };
        let right = if ignore_case { curr_str.to_lowercase() } else { curr_str.clone() };

        return match op {
            "any" | "none" | "any_case_sensitive" | "none_case_sensitive" => left == right,
            "str_starts_with_any" => left.starts_with(&right),
            "str_ends_with_any" => left.ends_with(&right),
            "str_contains_any" => left.contains(&right),
            "str_contains_none" => !left.contains(&right),
            _ => false
        };
    });

    if op == "none" || op == "none_case_sensitive" {
        return Some(!res);
    }

    Some(res)
}

trait ValueExt {
    fn as_number(&self) -> Option<f64>;
}

impl ValueExt for Value {
    fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }
}

