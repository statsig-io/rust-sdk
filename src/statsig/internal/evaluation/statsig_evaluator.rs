use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use serde_json::Value::{Null};
use crate::statsig::internal::evaluation::eval_helpers::{compare_str_with_regex, compare_time, value_to_string};

use crate::{StatsigOptions, StatsigUser, unwrap_or_return};

use super::country_lookup::CountryLookup;
use super::eval_helpers::{compare_numbers, compare_strings_in_array, compare_versions, compute_user_hash};
use super::eval_result::EvalResult;
use super::super::data_types::{APICondition, APIRule, APISpec};
use super::super::statsig_store::StatsigStore;
use super::ua_parser::UserAgentParser;

pub struct StatsigEvaluator {
    spec_store: Arc<StatsigStore>,
    country_lookup: CountryLookup,
    ua_parser: UserAgentParser,
}

impl StatsigEvaluator {
    pub fn new(spec_store: Arc<StatsigStore>, options: &StatsigOptions) -> StatsigEvaluator {
        StatsigEvaluator {
            spec_store,
            country_lookup: CountryLookup::new(),
            ua_parser: UserAgentParser::new(options.disable_user_agent_support),
        }
    }

    pub fn check_gate(&self, user: &StatsigUser, gate_name: &str) -> EvalResult {
        self.eval(user, gate_name, "gate")
    }

    pub fn get_config(&self, user: &StatsigUser, config_name: &str) -> EvalResult {
        self.eval(user, config_name, "config")
    }

    pub fn get_layer(&self, user: &StatsigUser, layer_name: &str) -> EvalResult {
        self.eval(user, layer_name, "layer")
    }
    
    fn eval(&self, user: &StatsigUser, spec_name: &str, spec_type: &str) -> EvalResult {
        self.spec_store.use_spec(spec_type, spec_name, |spec| {
            self.eval_spec(user, spec)
        })
    }

    fn eval_spec(&self, user: &StatsigUser, spec: Option<&APISpec>) -> EvalResult {
        let spec = match spec {
            Some(spec) => spec,
            _ => return EvalResult::default()
        };
        
        if !spec.enabled {
            return EvalResult {
                json_value: Some(spec.default_value.clone()),
                rule_id: "disabled".to_string(),
                ..EvalResult::default()
            };
        }

        let mut exposures: Vec<HashMap<String, String>> = vec![];

        for rule in spec.rules.iter() {
            let result = self.eval_rule(user, rule);

            if result.unsupported {
                return result;
            }

            if let Some(mut result_exposures) = result.secondary_exposures {
                exposures.append(&mut result_exposures);
            }

            if !result.bool_value {
                continue;
            }
            
            if let Some(delegated_result) = self.eval_delegate(user, rule, &exposures) {
                return delegated_result;
            }

            let pass = self.eval_pass_percentage(user, rule, &spec.salt);
            return EvalResult {
                bool_value: pass,
                json_value: match pass {
                    true => result.json_value,
                    false => Some(spec.default_value.clone())
                },
                rule_id: result.rule_id,
                secondary_exposures: Some(exposures.clone()),
                undelegated_secondary_exposures: Some(exposures),
                ..EvalResult::default()
            };
        }

        EvalResult {
            json_value: Some(spec.default_value.clone()),
            rule_id: "default".to_string(),
            secondary_exposures: Some(exposures.clone()),
            undelegated_secondary_exposures: Some(exposures),
            ..EvalResult::default()
        }
    }

    fn eval_rule(&self, user: &StatsigUser, rule: &APIRule) -> EvalResult {
        let mut exposures: Vec<HashMap<String, String>> = vec![];
        let mut pass = true;

        for condition in rule.conditions.iter() {
            let result = self.eval_condition(user, condition);
            if result.unsupported {
                return result;
            }

            if let Some(mut result_exposures) = result.secondary_exposures {
                exposures.append(&mut result_exposures);
            }

            if !result.bool_value {
                pass = false;
            }
        }

        EvalResult {
            bool_value: pass,
            json_value: Some(rule.return_value.clone()),
            rule_id: rule.id.clone(),
            secondary_exposures: Some(exposures),
            ..EvalResult::default()
        }
    }
    
    fn eval_delegate(&self, user: &StatsigUser, rule: &APIRule, exposures: &Vec<HashMap<String, String>>) -> Option<EvalResult> {
        let delegate = unwrap_or_return!(&rule.config_delegate, None);
        self.spec_store.use_spec("config", delegate, |spec| {
            let mut result = self.eval_spec(user, spec);
            if result.unsupported {
                return Some(result);
            }

            let undel_sec_expo = exposures.clone();
            let mut sec_expo = exposures.clone();
            if let Some(mut result_exposures) = result.secondary_exposures {
                sec_expo.append(&mut result_exposures);
            }
            
            let spec = unwrap_or_return!(spec, None);
            result.explicit_parameters = spec.explicit_parameters.clone();
            result.secondary_exposures = Some(sec_expo);
            result.undelegated_secondary_exposures = Some(undel_sec_expo);
            result.config_delegate = Some(delegate.clone());

            Some(result)
        })
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
            "user_bucket" => match self.get_hash_for_user_bucket(user, &condition) {
                Some(hash) => json!(hash),
                _ => Null
            },
            "unit_id" => json!(user.get_unit_id(&condition.id_type)),
            _ => return EvalResult::unsupported()
        };

        let operator = match &condition.operator {
            Some(operator) => operator.as_str(),
            None => return EvalResult::unsupported()
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

            _ => return EvalResult::unsupported(),
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
        let gate_name = value_to_string(target_value).expect("eval_nested_gate");
        let result = self.check_gate(user, &gate_name);

        if result.unsupported {
            return result;
        }

        let mut gate_value = result.bool_value;
        let exposure = HashMap::from([
            ("gate".to_string(), gate_name),
            ("gateValue".to_string(), gate_value.to_string()),
            ("ruleID".to_string(), result.rule_id)
        ]);

        if condition_type == "fail_gate" {
            gate_value = !gate_value;
        }

        let mut exposures = match result.secondary_exposures {
            Some(v) => v,
            None => vec![]
        };

        exposures.push(exposure);

        EvalResult {
            secondary_exposures: Some(exposures),
            ..EvalResult::boolean(gate_value)
        }
    }

    fn get_hash_for_user_bucket(&self, user: &StatsigUser, condition: &APICondition) -> Option<usize> {
        let unit_id = user.get_unit_id(&condition.id_type).unwrap_or("".to_string());
        let mut salt = "".to_string();
        if let Some(add_values) = &condition.additional_values {
            if let Value::String(s) = &add_values["salt"] {
                salt = s.clone();
            }
        }

        let hash = compute_user_hash(format!("{}.{}", salt, unit_id))?;
        Some(hash % 1000)
    }
}
