use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use serde_json::Value::Null;

use crate::statsig::internal::evaluation::eval_helpers::compute_user_hash;
use crate::statsig::internal::evaluation::ua_parser::UserAgentParser;
use crate::StatsigUser;

use super::country_lookup::CountryLookup;
use super::eval_helpers::{compare_numbers, match_string_in_array};
use super::eval_result::EvalResult;
use super::super::data_types::{APICondition, APIRule, APISpec};
use super::super::store::StatsigStore;

pub struct StatsigEvaluator {
    spec_store: Arc<Mutex<StatsigStore>>,
    country_lookup: CountryLookup,
    ua_parser: UserAgentParser,
}

impl StatsigEvaluator {
    pub fn new(spec_store: Arc<Mutex<StatsigStore>>) -> StatsigEvaluator {
        StatsigEvaluator {
            spec_store,
            country_lookup: CountryLookup::new(),
            ua_parser: UserAgentParser::new(),
        }
    }

    pub async fn check_gate(&mut self, user: &StatsigUser, gate_name: &String) -> EvalResult {
        let store = self.spec_store.lock().unwrap();
        let gate = store.get_gate(gate_name);

        if gate_name == "test_country" {
            println!("foo")
        }

        match gate {
            Some(spec) => self.eval_spec(user, spec),
            None => EvalResult::default()
        }
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
        let condition_type = condition.condition_type.to_lowercase();
        let maybe_value = match condition_type.as_str() {
            "public" => return EvalResult::boolean(true),
            "user_field" => user.get_user_value(&condition.field),
            "ip_based" => match user.get_user_value(&condition.field) {
                Null => self.country_lookup.get_value_from_ip(user, &condition.field),
                v => v
            },
            "ua_based" => match user.get_user_value(&condition.field) {
                Null => self.ua_parser.get_value_from_user_agent(user, &condition.field),
                v => v
            },
            "current_time" => json!(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()),
            _ => Null
        };

        let value = match maybe_value {
            Null => return EvalResult::fetch_from_server(),
            v => v,
        };

        let target_value = json!(condition.target_value);
        let operator = match &condition.operator {
            Some(operator) => operator.as_str(),
            None => return EvalResult::fetch_from_server()
        };

        let result = match operator {
            // numerical comparison
            "gt" | "gte" | "lt" | "lte" =>
                compare_numbers(&value, &target_value, operator)
                    .unwrap_or(false),

            "any" | "none" =>
                match_string_in_array(&value, &target_value, true, operator)
                    .unwrap_or(false),

            "any_case_sensitive" | "none_case_sensitive" =>
                match_string_in_array(&value, &target_value, false, operator)
                    .unwrap_or(false),

            // string comparison
            "str_starts_with_any" | "str_ends_with_any" | "str_contains_any" | "str_contains_none" =>
                match_string_in_array(&value, &target_value, true, operator)
                    .unwrap_or(false),

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
}

