use chrono::Utc;
use std::collections::{hash_map::RandomState, HashMap};
use std::string::ToString;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{StatsigEvent, StatsigUser};

use super::EvalResult;

type StatsigEnvironment = Option<HashMap<String, String>>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsigEventInternal {
    #[serde(flatten)]
    pub event_data: StatsigEvent,

    pub user: StatsigUser,
    pub time: u64,
    pub secondary_exposures: Option<Vec<HashMap<String, String>>>,
}

pub(crate) fn make_gate_exposure(
    user: &StatsigUser,
    gate_name: &str,
    eval_result: &EvalResult,
    statsig_environment: &StatsigEnvironment,
) -> StatsigEventInternal {
    let mut metatadata = make_metadata_for_exposure("gate", gate_name, eval_result);
    metatadata.extend(HashMap::from([(
        "gateValue".to_string(),
        json!(eval_result.bool_value.to_string()),
    )]));

    let event = StatsigEvent {
        event_name: "statsig::gate_exposure".to_string(),
        value: None,
        metadata: Some(metatadata),
    };

    finalize_with_cloned_or_empty_exposures(
        user,
        event,
        statsig_environment,
        &eval_result.secondary_exposures,
    )
}

pub(crate) fn make_config_exposure(
    user: &StatsigUser,
    config_name: &str,
    eval_result: &EvalResult,
    statsig_environment: &StatsigEnvironment,
) -> StatsigEventInternal {
    let event = StatsigEvent {
        event_name: "statsig::config_exposure".to_string(),
        value: None,
        metadata: Some(make_metadata_for_exposure(
            "config",
            config_name,
            eval_result,
        )),
    };

    finalize_with_cloned_or_empty_exposures(
        user,
        event,
        statsig_environment,
        &eval_result.secondary_exposures,
    )
}

pub(crate) fn make_layer_exposure(
    user: &StatsigUser,
    layer_name: &str,
    parameter_name: &str,
    eval_result: &EvalResult,
    statsig_environment: &StatsigEnvironment,
) -> StatsigEventInternal {
    let mut exposures = &eval_result.undelegated_secondary_exposures;
    let mut allocated_experiment = None;
    let is_explicit = match &eval_result.explicit_parameters {
        Some(explicit_params) => explicit_params.iter().any(|x| x == parameter_name),
        _ => false,
    };

    if is_explicit {
        allocated_experiment = eval_result.config_delegate.clone();
        exposures = &eval_result.secondary_exposures;
    }
    let mut metadata = make_metadata_for_exposure("config", layer_name, eval_result);
    metadata.extend(HashMap::from([
        (
            "allocatedExperiment".to_string(),
            json!(allocated_experiment.unwrap_or("".to_string())),
        ),
        ("parameterName".to_string(), json!(parameter_name)),
        (
            "isExplicitParameter".to_string(),
            json!(format!("{}", is_explicit)),
        ),
    ]));
    let event = StatsigEvent {
        event_name: "statsig::layer_exposure".to_string(),
        value: None,
        metadata: Some(metadata),
    };

    finalize_with_cloned_or_empty_exposures(user, event, statsig_environment, exposures)
}

pub(crate) fn finalize_event(
    user: &StatsigUser,
    event: StatsigEvent,
    statsig_environment: &StatsigEnvironment,
) -> StatsigEventInternal {
    finalize_with_optional_exposures(user, event, statsig_environment, None)
}

fn finalize_with_cloned_or_empty_exposures(
    user: &StatsigUser,
    event: StatsigEvent,
    statsig_environment: &StatsigEnvironment,
    secondary_exposures: &Option<Vec<HashMap<String, String>>>,
) -> StatsigEventInternal {
    let exposures = match secondary_exposures {
        Some(expo) => expo.clone(),
        None => vec![],
    };

    finalize_with_optional_exposures(user, event, statsig_environment, Some(exposures))
}

fn finalize_with_optional_exposures(
    user: &StatsigUser,
    event: StatsigEvent,
    statsig_environment: &StatsigEnvironment,
    secondary_exposures: Option<Vec<HashMap<String, String>>>,
) -> StatsigEventInternal {
    let mut user_copy = user.clone();

    if let Some(env) = statsig_environment {
        user_copy.statsig_environment = Some(env.clone());
    }

    user_copy.private_attributes = None;

    StatsigEventInternal {
        event_data: event,
        user: user_copy,
        time: Utc::now().timestamp_millis() as u64,
        secondary_exposures,
    }
}

fn make_metadata_for_exposure(
    config_key: &str,
    config_name: &str,
    eval_result: &EvalResult,
) -> HashMap<String, Value, RandomState> {
    HashMap::from([
        (config_key.to_string(), json!(config_name)),
        ("ruleID".to_string(), json!(eval_result.rule_id)),
        (
            "configSyncTime".to_string(),
            json!(eval_result.evaluation_details.config_sync_time),
        ),
        (
            "reason".to_string(),
            json!(eval_result.evaluation_details.reason),
        ),
        (
            "initTime".to_string(),
            json!(eval_result.evaluation_details.init_time),
        ),
        (
            "serverTime".to_string(),
            json!(eval_result.evaluation_details.server_time),
        ),
    ])
}
