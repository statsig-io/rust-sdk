use serde::de::DeserializeOwned;
use serde_json::{from_value, Value};
use std::collections::HashMap;

pub struct DynamicConfig {
    pub name: String,
    pub value: HashMap<String, Value>,
    pub rule_id: String,
}

impl DynamicConfig {
    pub fn get<T: DeserializeOwned>(&self, key: &str, default: T) -> T {
        if !self.value.contains_key(key) {
            return default;
        }

        if let Ok(value) = from_value(self.value[key].clone()) {
            return value;
        }

        default
    }
}
