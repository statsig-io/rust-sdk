use crate::StatsigUser;
use serde_json::Value::Null;
use serde_json::{json, Value};

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
        let field = match field {
            Some(f) => f,
            _ => return Null,
        };

        let str_value = match field.to_lowercase().as_str() {
            "userid" | "user_id" => &self.user_id,
            "email" => &self.email,
            "ip" => &self.ip,
            "useragent" | "user_agent" => &self.user_agent,
            "country" => &self.country,
            "locale" => &self.locale,
            "appversion" | "app_version" => &self.app_version,
            _ => &None,
        };

        if let Some(value) = str_value {
            return json!(value);
        }

        if let Some(custom) = &self.custom {
            if let Some(custom_value) = custom.get(field.as_str()) {
                return custom_value.clone();
            }
            if let Some(custom_value) = custom.get(field.to_uppercase().to_lowercase().as_str()) {
                return custom_value.clone();
            }
        }

        if let Some(private_attributes) = &self.private_attributes {
            if let Some(private_value) = private_attributes.get(field.as_str()) {
                return private_value.clone();
            }
            if let Some(private_value) =
                private_attributes.get(field.to_uppercase().to_lowercase().as_str())
            {
                return private_value.clone();
            }
        }

        return Null;
    }

    pub fn get_value_from_environment(&self, field: &Option<String>) -> Value {
        let field_lowered = match field {
            Some(f) => f.to_lowercase(),
            _ => return Null,
        };

        let env = match &self.statsig_environment {
            Some(e) => e,
            _ => return Null,
        };

        for key in env.keys() {
            if key.to_lowercase() == field_lowered {
                return json!(env[key]);
            }
        }

        Null
    }
}
