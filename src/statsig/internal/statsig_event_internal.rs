use std::collections::HashMap;
use std::string::ToString;

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

    finalize_with_exposures(event, statsig_environment, eval_result.exposures.clone())
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

    finalize_with_exposures(event, statsig_environment, eval_result.exposures.clone())
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
        time: 1,
        secondary_exposures: match secondary_exposures {
            Some(x) => x,
            _ => vec![]
        },
    }
}
