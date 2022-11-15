use std::collections::HashMap;
use std::string::ToString;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{StatsigEvent, StatsigUser};

use super::EvalResult;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsigEventInternal {
    #[serde(flatten)]
    pub event_data: StatsigEvent,

    pub time: u64,
    pub secondary_exposures: Vec<HashMap<String, String>>,
}

pub(crate) fn make_gate_exposure(user: StatsigUser, gate_name: &String, eval_result: &EvalResult, statsig_environment: &Option<HashMap<String, String>>) -> StatsigEventInternal {
    let mut event = finalize_event(
        StatsigEvent {
            user,
            event_name: "statsig::gate_exposure".to_string(),
            value: None,
            metadata: Some(HashMap::from([
                ("gate".to_string(), json!(gate_name)),
                ("gateValue".to_string(), json!(eval_result.bool_value.to_string())),
                ("ruleID".to_string(), json!(eval_result.rule_id))
            ])),
        },
        statsig_environment,
    );

    event.secondary_exposures = match eval_result.exposures.clone() {
        Some(v) => v,
        None => vec![]
    };

    event
}

pub(crate) fn finalize_event(mut event: StatsigEvent, statsig_environment: &Option<HashMap<String, String>>) -> StatsigEventInternal {
    if let Some(env) = statsig_environment {
        event.user.statsig_environment = Some(env.clone());
    }
    event.user.private_attributes = None;

    StatsigEventInternal {
        event_data: event,
        time: 1,
        secondary_exposures: vec![],
    }
}
