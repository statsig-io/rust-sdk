use serde_json::Value;
use std::collections::HashMap;

use super::eval_details::{EvalDetails, EvaluationReason};

pub struct EvalResult {
    pub bool_value: bool,
    pub json_value: Option<Value>,
    pub rule_id: String,
    pub unsupported: bool,
    pub secondary_exposures: Option<Vec<HashMap<String, String>>>,
    pub undelegated_secondary_exposures: Option<Vec<HashMap<String, String>>>,
    pub explicit_parameters: Option<Vec<String>>,
    pub config_delegate: Option<String>,
    pub is_experiment_group: bool,
    pub evaluation_details: EvalDetails,
    pub group_name: Option<String>,
}

impl EvalResult {
    pub fn unsupported() -> Self {
        Self {
            unsupported: true,
            rule_id: "unsupported".to_string(),
            ..Self::default()
        }
    }

    pub fn boolean(bool_value: bool) -> Self {
        Self {
            bool_value,
            ..Self::default()
        }
    }

    pub fn unrecognized(mut eval_details: EvalDetails) -> Self {
        eval_details.reason = EvaluationReason::Unrecognized;
        Self {
            rule_id: "default".to_string(),
            evaluation_details: eval_details,
            ..Self::default()
        }
    }

    pub fn uninitialized(mut eval_details: EvalDetails) -> Self {
        eval_details.reason = EvaluationReason::Uninitialized;
        Self {
            rule_id: "default".to_string(),
            evaluation_details: eval_details,
            ..Self::default()
        }
    }

    pub fn default() -> Self {
        Self {
            bool_value: false,
            json_value: None,
            rule_id: "".to_string(),
            unsupported: false,
            secondary_exposures: None,
            undelegated_secondary_exposures: None,
            explicit_parameters: None,
            config_delegate: None,
            is_experiment_group: false,
            evaluation_details: EvalDetails::default(),
            group_name: None,
        }
    }
}
