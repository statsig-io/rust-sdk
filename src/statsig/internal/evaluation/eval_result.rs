use std::collections::HashMap;
use serde_json::Value;

pub struct EvalResult {
    pub bool_value: bool,
    pub json_value: Option<Value>,
    pub rule_id: String,
    pub fetch_from_server: bool,
    pub secondary_exposures: Option<Vec<HashMap<String, String>>>,
    pub undelegated_secondary_exposures: Option<Vec<HashMap<String, String>>>,
    pub explicit_parameters: Option<Vec<String>>,
    pub config_delegate: Option<String>
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
            secondary_exposures: None,
            undelegated_secondary_exposures: None,
            explicit_parameters: None,
            config_delegate: None
        }
    }
}