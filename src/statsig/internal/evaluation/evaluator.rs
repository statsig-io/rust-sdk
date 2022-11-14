use std::borrow::{BorrowMut, Cow};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use serde_json::Value::{Null};
use crate::statsig::internal::evaluation::eval_helpers::{compare_str_with_regex, compare_time, value_to_string};

use crate::StatsigUser;

use super::country_lookup::CountryLookup;
use super::eval_helpers::{compare_numbers, compare_strings_in_array, compare_versions, compute_user_hash};
use super::eval_result::EvalResult;
use super::super::data_types::{APICondition, APIRule, APISpec};
use super::super::store::StatsigStore;
use super::ua_parser::UserAgentParser;

pub struct StatsigEvaluator {
    spec_store: Arc<StatsigStore>,
    country_lookup: CountryLookup,
    ua_parser: UserAgentParser,
}

impl StatsigEvaluator {
    pub fn new(spec_store: Arc<StatsigStore>) -> StatsigEvaluator {
        StatsigEvaluator {
            spec_store,
            country_lookup: CountryLookup::new(),
            ua_parser: UserAgentParser::new(),
        }
    }

    pub fn check_gate(&self, user: &StatsigUser, gate_name: &String) -> EvalResult {
        self.spec_store.use_gate(gate_name, |gate| {
            match gate {
                Some(gate) => self.eval_spec(user, gate),
                _ => EvalResult::default()
            }
        })
        // let gate = self.spec_store.get_gate(gate_name).unwrap();
        // self.eval_spec(user, &gate).await
        // match self.spec_store.get_gate(gate_name) {
        //     Some(gate) => self.eval_spec(user, &gate).await,
        //     _ => EvalResult::default()
        // }
    }

    fn eval_spec(&self, user: &StatsigUser, spec: &APISpec) -> EvalResult {
        if !spec.enabled {
            return EvalResult {
                rule_id: "disabled".to_string(),
                ..EvalResult::default()
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
            return EvalResult {
                bool_value: pass,
                ..EvalResult::default()
            };
        }


        EvalResult::default()
    }

    fn eval_rule(&self, user: &StatsigUser, rule: &APIRule) -> EvalResult {
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

        EvalResult {
            bool_value: pass,
            exposures: Some(exposures),
            ..EvalResult::default()
        }
    }

    fn eval_condition(&self, user: &StatsigUser, condition: &APICondition) -> EvalResult {
        let target_value = json!(condition.target_value);
        let condition_type = condition.condition_type.to_lowercase();

        let value = match condition_type.as_str() {
            "public" => return EvalResult::boolean(true),
            "fail_gate" | "pass_gate" => return self.eval_nested_gate(user, &target_value, &condition_type),
            "ip_based" => match user.get_user_value(&condition.field) {
                Null => self.country_lookup.get_value_from_ip(user, &condition.field),
                v => v
            },
            "ua_based" => match user.get_user_value(&condition.field) {
                Null => self.ua_parser.get_value_from_user_agent(user, &condition.field),
                v => v
            },
            "user_field" => user.get_user_value(&condition.field),
            "environment_field" => user.get_value_from_environment(&condition.field),
            "current_time" => match SystemTime::now().duration_since(UNIX_EPOCH).ok() {
                Some(time) => json!(time.as_millis().to_string()),
                _ => Null
            },
            "user_bucket" => json!(self.get_hash_for_user_bucket(user, &condition)),
            "unit_id" => json!(user.get_unit_id(&condition.id_type)),
            _ => return EvalResult::fetch_from_server()
        };

        let operator = match &condition.operator {
            Some(operator) => operator.as_str(),
            None => return EvalResult::fetch_from_server()
        };

        let result = match operator {
            // numerical comparison
            "gt" | "gte" | "lt" | "lte" =>
                compare_numbers(&value, &target_value, operator)
                    .unwrap_or(false),

            // version comparison
            "version_gt" | "version_gte" | "version_lt" | "version_lte" | "version_eq" | "version_neq" =>
                compare_versions(&value, &target_value, operator)
                    .unwrap_or(false),

            // string/array comparison
            "any" | "none" | "str_starts_with_any" | "str_ends_with_any" | "str_contains_any" | "str_contains_none" =>
                compare_strings_in_array(&value, &target_value, operator, true),
            "any_case_sensitive" | "none_case_sensitive" =>
                compare_strings_in_array(&value, &target_value, operator, false),
            "str_matches" => compare_str_with_regex(&value, &target_value),

            // time comparison
            "before" | "after" | "on" =>
                compare_time(&value, &target_value, operator)
                    .unwrap_or(false),

            "eq" => value == target_value,
            "neq" => value != target_value,

            _ => return EvalResult::fetch_from_server(),
        };
        return EvalResult::boolean(result);
    }

    fn eval_pass_percentage(&self, user: &StatsigUser, rule: &APIRule, spec_salt: &String) -> bool {
        let rule_salt = rule.salt.as_ref().unwrap_or(&rule.id);
        let unit_id = user.get_unit_id(&rule.id_type).unwrap_or("".to_string());
        match compute_user_hash(format!("{}.{}.{}", spec_salt, rule_salt, unit_id)) {
            Some(hash) => ((hash % 10000) as f64) < rule.pass_percentage * 100.0,
            None => false
        }
    }

    fn eval_nested_gate(&self, user: &StatsigUser, target_value: &Value, condition_type: &String) -> EvalResult {
        let gate_name = value_to_string(target_value).unwrap();
        let result = self.check_gate(user, &gate_name);

        if result.fetch_from_server {
            return result;
        }

        if condition_type == "pass_gate" {
            return result;
        }

        EvalResult::boolean(!result.bool_value)
    }

    fn get_hash_for_user_bucket(&self, user: &StatsigUser, condition: &APICondition) -> usize {
        let unit_id = user.get_unit_id(&condition.id_type).unwrap_or("".to_string());
        let mut salt = "".to_string();
        if let Some(add_values) = &condition.additional_values {
            if let Value::String(s) = &add_values["salt"] {
                salt = s.clone();
            }
        }

        let hash = match compute_user_hash(format!("{}.{}", salt, unit_id)) {
            Some(hash) => hash,
            _ => 0
        };
        hash % 1000
    }
}
