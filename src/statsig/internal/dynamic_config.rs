use std::collections::HashMap;
use serde_json::{Value, from_value};
use serde::de::DeserializeOwned;

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

        return default;
    }
}

