use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::format;
use std::mem::size_of;
use std::iter::Map;
use std::ops::Deref;
use std::ptr::null;
use std::rc::Weak;
use std::sync::{Arc, Mutex, RwLock};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Digest, Sha256};
use sha2::digest::generic_array::sequence::Split;
use crate::country_lookup::CountryLookup;

use crate::data_types::{APICondition, APIRule, APISpec};
use crate::eval_helpers::{compare_numbers, match_string_in_array};
use crate::statsig_store::StatsigStore;
use crate::StatsigUser;

pub struct StatsigEvaluator {
    pub spec_store: Arc<Mutex<StatsigStore>>,
    country_lookup: CountryLookup,
}

pub struct SpecEval {
    pub bool_value: bool,
    json_value: Option<Map<String, Value>>,
    rule_id: String,
    fetch_from_server: bool,
    exposures: Option<Vec<HashMap<String, String>>>,
}

impl SpecEval {
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

impl StatsigEvaluator {
    pub fn new(spec_store: Arc<Mutex<StatsigStore>>) -> StatsigEvaluator {
        StatsigEvaluator {
            spec_store,
            country_lookup: CountryLookup::new(),
        }
    }

    pub async fn check_gate(&mut self, user: &StatsigUser, gate_name: &String) -> SpecEval {
        let mut store = self.spec_store.lock().unwrap();
        let gate = store.get_gate(gate_name);
        match gate {
            Some(spec) => self.eval_spec(user, spec),
            None => SpecEval::default()
        }
    }

    fn eval_spec(&self, user: &StatsigUser, spec: &APISpec) -> SpecEval {
        if !spec.enabled {
            return SpecEval {
                rule_id: "disabled".to_string(),
                ..SpecEval::default()
            };
        }

        let mut exposures: Vec<HashMap<String, String>> = vec![];

        for rule in spec.rules.iter() {
            let result = self.eval_rule(user, rule);

            if result.fetch_from_server {
                return result;
            }

            if let Some(mut result_exposures) = result.exposures {
                exposures.append(&mut result_exposures);
            }

            if !result.bool_value {
                continue;
            }

            let pass = self.eval_pass_percentage(user, rule, &spec.salt);
            return SpecEval {
                bool_value: pass,
                ..SpecEval::default()
            };
        }


        SpecEval::default()
    }

    fn eval_rule(&self, user: &StatsigUser, rule: &APIRule) -> SpecEval {
        let mut exposures: Vec<HashMap<String, String>> = vec![];
        let mut pass = true;

        for condition in rule.conditions.iter() {
            let result = self.eval_condition(user, condition);
            if result.fetch_from_server {
                return result;
            }

            if let Some(mut result_exposures) = result.exposures {
                exposures.append(&mut result_exposures);
            }

            if !result.bool_value {
                pass = false;
            }
        }

        SpecEval {
            bool_value: pass,
            exposures: Some(exposures),
            ..SpecEval::default()
        }
    }

    fn eval_condition(&self, user: &StatsigUser, condition: &APICondition) -> SpecEval {
        let condition_type = condition.condition_type.to_lowercase();
        let opt_value = match condition_type.as_str() {
            "public" => return SpecEval::boolean(true),
            "user_field" => user.get_user_value(&condition.field),
            "ip_based" => user.get_user_value(&condition.field).or(self.get_value_from_ip(user, &condition.field)),
            "current_time" => Some(json!(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis())),
            _ => None
        };

        let value = match opt_value {
            Some(v) => v,
            None => return SpecEval::fetch_from_server()
        };

        let target_value = json!(condition.target_value);
        let operator = match &condition.operator {
            Some(operator) => operator.as_str(),
            None => ""
        };

        let result = match operator {
            // numerical comparison
            "gt" | "gte" | "lt" | "lte" =>
                compare_numbers(&value, &target_value, operator)
                    .unwrap_or(false),

            // string comparison
            "str_starts_with_any" | "str_ends_with_any" | "str_contains_any" | "str_contains_none" =>
                match_string_in_array(&value, &target_value, true, operator)
                    .unwrap_or(false),

            _ => return SpecEval::fetch_from_server(),
        };
        return SpecEval::boolean(result);
    }

    fn eval_pass_percentage(&self, user: &StatsigUser, rule: &APIRule, spec_salt: &String) -> bool {
        let rule_salt = rule.salt.as_ref().unwrap_or(&rule.id);
        let unit_id = user.get_unit_id(&rule.id_type).unwrap_or("".to_string());
        match self.compute_user_hash(format!("{}.{}.{}", spec_salt, rule_salt, unit_id)) {
            Some(hash) => ((hash % 10000) as f64) < rule.pass_percentage * 100.0,
            None => false
        }
    }

    fn compute_user_hash(&self, value: String) -> Option<usize> {
        let mut sha256 = Sha256::new();
        sha256.update(value.as_str().as_bytes());
        let result = sha256.finalize();
        match result.split_at(size_of::<usize>()).0.try_into() {
            Ok(bytes) => Some(usize::from_be_bytes(bytes)),
            _ => None
        }
    }

    fn get_value_from_ip(&self, user: &StatsigUser, field: &Option<String>) -> Option<Value> {
        let unwrapped_field = match field {
            Some(f) => f.as_str(),
            _ => return None
        };

        if unwrapped_field != "country" {
            return None;
        }

        let ip = match user.get_user_value_with_str(&"ip") {
            Some(ip) => ip.to_string(),
            _ => return None
        };

        let cc = self.country_lookup.lookup(&ip);
        return None;
    }
}

impl StatsigUser {
    fn get_unit_id(&self, id_type: &String) -> Option<String> {
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

    fn get_user_value(&self, field: &Option<String>) -> Option<Value> {
        match field {
            Some(f) => self.get_user_value_with_str(f),
            _ => None
        }
    }

    fn get_user_value_with_str(&self, field: &str) -> Option<Value> {
        Some(json!(match field.to_lowercase().as_str() {
            "userid" | "user_id" => self.user_id.clone(),
            "email" => self.email.clone(),
            "ip" => None,
            "useragent" | "user_agent" => None,
            "country" => None,
            "locale" => None,
            "appversion" | "app_version" => None,
            "custom" => None,
            "privateattributes" | "private_attributes" => None,
            _ => None
        }))
    }
}
