use serde_json::{json, Value};
use serde_json::Value::Null;
use crate::StatsigUser;

impl StatsigUser {
    pub fn get_unit_id(&self, id_type: &String) -> Option<String> {
        if id_type.to_lowercase() == *"userid" {
            return self.user_id.clone();
        }

        let custom_ids = match &self.custom_ids {
            Some(x) => x,
            None => return None,
        };

        if let Some(custom_id) = custom_ids.get(id_type) {
            return Some(custom_id.clone());
        }

        if let Some(custom_id) = custom_ids.get(id_type.to_lowercase().as_str()) {
            return Some(custom_id.clone());
        }

        return None;
    }

    pub fn get_user_value(&self, field: &Option<String>) -> Value {
        let field_lowered = match field {
            Some(f) => f.to_lowercase(),
            _ => return Null
        };

        let str_value = match field_lowered.as_str() {
            "userid" | "user_id" => &self.user_id,
            "email" => &self.email,
            "ip" => &self.ip,
            "useragent" | "user_agent" => &self.user_agent,
            "country" => &self.country,
            "locale" => &self.locale,
            "appversion" | "app_version" => &self.app_version,
            _ => &None
        };

        if let Some(value) = str_value {
            return json!(value);
        }

        let dict_value = match field_lowered.as_str() {
            "custom" => &self.custom,
            "privateattributes" | "private_attributes" => &self.private_attributes,
            _ => &None
        };

        if let Some(value) = dict_value {
            return json!(value);
        }

        return Null;
    }
}
