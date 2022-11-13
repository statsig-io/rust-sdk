use std::collections::HashMap;
use std::iter::Map;
use serde_json::Value;

pub struct EvalResult {
    pub bool_value: bool,
    pub json_value: Option<Map<String, Value>>,
    pub rule_id: String,
    pub fetch_from_server: bool,
    pub exposures: Option<Vec<HashMap<String, String>>>,
}

impl EvalResult {
    pub fn fetch_from_server() -> Self {
        Self {
            fetch_from_server: true,
            ..Self::default()
        }
    }

    pub fn boolean(bool_value: bool) -> Self {
        Self {
            bool_value,
            ..Self::default()
        }
    }

    pub fn default() -> Self {
        Self {
            bool_value: false,
            json_value: None,
            rule_id: "".to_string(),
            fetch_from_server: false,
            exposures: None,
        }
    }
}