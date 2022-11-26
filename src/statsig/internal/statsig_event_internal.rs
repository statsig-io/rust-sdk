use std::collections::HashMap;
use std::string::ToString;
use chrono::Utc;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{StatsigEvent, StatsigUser};

use super::EvalResult;

type StatsigEnvironment = Option<HashMap<String, String>>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsigEventInternal {
    #[serde(flatten)]
    pub event_data: StatsigEvent,

    pub time: u64,
    pub secondary_exposures: Vec<HashMap<String, String>>,
}

pub(crate) fn make_gate_exposure(
    user: StatsigUser,
    gate_name: &String,
    eval_result: &EvalResult,
    statsig_environment: &StatsigEnvironment) -> StatsigEventInternal {
    let event = StatsigEvent {
        user,
        event_name: "statsig::gate_exposure".to_string(),
        value: None,
        metadata: Some(HashMap::from([
            ("gate".to_string(), json!(gate_name)),
            ("gateValue".to_string(), json!(eval_result.bool_value.to_string())),
            ("ruleID".to_string(), json!(eval_result.rule_id))
        ])),
    };

    finalize_with_exposures(event, statsig_environment, eval_result.secondary_exposures.clone())
}

pub(crate) fn make_config_exposure(
    user: StatsigUser,
    config_name: &String,
    eval_result: &EvalResult,
    statsig_environment: &StatsigEnvironment) -> StatsigEventInternal {
    let event = StatsigEvent {
        user,
        event_name: "statsig::config_exposure".to_string(),
        value: None,
        metadata: Some(HashMap::from([
            ("config".to_string(), json!(config_name)),
            ("ruleID".to_string(), json!(eval_result.rule_id))
        ])),
    };

    finalize_with_exposures(event, statsig_environment, eval_result.secondary_exposures.clone())
}

pub(crate) fn make_layer_exposure(
    user: StatsigUser,
    layer_name: &String,
    parameter_name: &String,
    eval_result: &EvalResult,
    statsig_environment: &StatsigEnvironment
) -> StatsigEventInternal {
    
    let mut exposures = &eval_result.undelegated_secondary_exposures;
    let mut allocated_experiment = None;
    let is_explicit = match &eval_result.explicit_parameters {
        Some(explicit_params) => explicit_params.contains(parameter_name),
        _ => false
    };
    
    if is_explicit {
        allocated_experiment = eval_result.config_delegate.clone();
        exposures = &eval_result.secondary_exposures;
    }
    
    let event = StatsigEvent {
        user,
        event_name: "statsig::layer_exposure".to_string(),
        value: None,
        metadata: Some(HashMap::from([
            ("config".to_string(), json!(layer_name)),
            ("ruleID".to_string(), json!(eval_result.rule_id)),
            ("allocatedExperiment".to_string(), json!(allocated_experiment.unwrap_or("".to_string()))),
            ("parameterName".to_string(), json!(parameter_name)),
            ("isExplicitParameter".to_string(), json!(format!("{}", is_explicit))),
        ])),
    };

    finalize_with_exposures(event, statsig_environment, exposures.clone())
}

pub(crate) fn finalize_event(event: StatsigEvent, statsig_environment: &StatsigEnvironment) -> StatsigEventInternal {
    finalize_with_exposures(event, statsig_environment, None)
}

fn finalize_with_exposures(mut event: StatsigEvent, statsig_environment: &StatsigEnvironment, secondary_exposures: Option<Vec<HashMap<String, String>>>) -> StatsigEventInternal {
    if let Some(env) = statsig_environment {
        event.user.statsig_environment = Some(env.clone());
    }
    event.user.private_attributes = None;

    StatsigEventInternal {
        event_data: event,
        time: Utc::now().timestamp_millis() as u64,
        secondary_exposures: match secondary_exposures {
            Some(x) => x,
            _ => vec![]
        },
    }
}
