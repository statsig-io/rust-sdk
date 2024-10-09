use std::collections::HashMap;

use base64;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde_json::Value::Null;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::statsig::internal::data_types::APISpec;
use crate::statsig::internal::statsig_store::StatsigStore;
use crate::statsig::internal::EvalResult;
use crate::{unwrap_or_noop, unwrap_or_return, StatsigUser};

type SecondaryExposures = Option<Vec<HashMap<String, String>>>;

pub struct ClientInitResponseFormatter {}

impl ClientInitResponseFormatter {
    pub fn get_formatted_response(
        eval_func: impl Fn(&StatsigUser, &APISpec) -> EvalResult,
        user: &StatsigUser,
        spec_store: &StatsigStore,
    ) -> Value {
        let specs = unwrap_or_return!(spec_store.specs.read().ok(), Null);

        let get_evaluated_spec = |spec_name, spec| -> Option<Value> {
            let eval_result = eval_func(user, spec);
            let mut result = HashMap::from([
                ("name".to_string(), json!(hash_name(spec_name))),
                ("rule_id".to_string(), json!(eval_result.rule_id)),
                (
                    "secondary_exposures".to_string(),
                    json!(clean_exposures(&eval_result.secondary_exposures)),
                ),
            ]);

            match spec._type.as_str() {
                "feature_gate" => {
                    if spec.entity == "segment" || spec.entity == "holdout" {
                        return None;
                    }
                    result.insert("value".into(), json!(eval_result.bool_value));
                    result.insert("id_type".into(), json!(spec.id_type));
                }

                "dynamic_config" => {
                    result.insert("value".into(), json!(eval_result.json_value));
                    result.insert("group".into(), json!(eval_result.rule_id));
                    result.insert(
                        "is_device_based".into(),
                        json!(spec.id_type.to_lowercase() == "stableid"),
                    );
                    if !eval_result.group_name.is_none() {
                        result.insert("group_name".into(), json!(eval_result.group_name));
                    }
                    if spec.entity.as_str() != "layer" {
                        result.insert("id_type".into(), json!(spec.id_type));
                    }

                    match spec.entity.as_str() {
                        "experiment" => {
                            populate_experiment_fields(spec, &eval_result, &mut result, spec_store)
                        }
                        "layer" => populate_layer_fields(
                            spec,
                            &eval_result,
                            &mut result,
                            spec_store,
                            |delegate_spec| eval_func(user, delegate_spec),
                        ),
                        _ => { /* noop */ }
                    }
                }
                _ => return None,
            }

            Some(json!(result))
        };

        let eval_all = |spec_type: &str| -> HashMap<String, Option<Value>> {
            let iter = match spec_type {
                "gates" => specs.gates.iter(),
                "configs" => specs.configs.iter(),
                "layers" => specs.layers.iter(),
                _ => return HashMap::new(),
            };

            iter.map(|(name, spec)| (hash_name(name), get_evaluated_spec(name, spec)))
                .filter(|(_, result)| result.is_some())
                .collect()
        };

        let mut evaluated_keys: HashMap<String, Value> = HashMap::new();

        if let Some(user_id) = &user.user_id {
            evaluated_keys.insert("userID".into(), json!(user_id));
        }
        if let Some(custom_ids) = &user.custom_ids {
            evaluated_keys.insert("customIDs".into(), json!(custom_ids));
        }

        json!({
            "feature_gates": eval_all("gates"),
            "dynamic_configs": eval_all("configs"),
            "layer_configs": eval_all("layers"),
            "evaluated_keys": json!(evaluated_keys),
            "sdkParams": json!({}),
            "generator": "statsig-rust-sdk",
            "has_updates": true,
            "time": 0,
        })
    }
}

fn hash_name(name: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(name.as_bytes());

    BASE64_STANDARD.encode(hash.finalize())
}

fn clean_exposures(_exposures: &SecondaryExposures) -> SecondaryExposures {
    Some(vec![])
}

fn populate_experiment_fields(
    spec: &APISpec,
    eval_result: &EvalResult,
    result: &mut HashMap<String, Value>,
    spec_store: &StatsigStore,
) {
    result.insert(
        "is_user_in_experiment".into(),
        json!(eval_result.is_experiment_group),
    );
    result.insert(
        "is_experiment_active".into(),
        json!(spec.is_active.unwrap_or(false)),
    );

    if !spec.has_shared_params.unwrap_or(false) {
        return;
    }

    result.insert("is_in_layer".into(), json!(true));
    result.insert(
        "explicit_parameters".into(),
        json!(spec.explicit_parameters.as_ref().unwrap_or(&vec![])),
    );

    let layer_name = match spec_store.get_layer_name_for_experiment(&spec.name) {
        Some(layer_name) => layer_name,
        None => return,
    };
    let layer_value = spec_store.use_spec("layer", layer_name.as_str(), |layer, _eval_details| {
        if let Some(layer_value) = layer {
            return layer_value.default_value.clone();
        }

        Null
    });

    let merged = merge_json_value(&layer_value, json!(eval_result.json_value));
    result.insert("value".into(), merged);
}

fn populate_layer_fields(
    spec: &APISpec,
    eval_result: &EvalResult,
    result: &mut HashMap<String, Value>,
    spec_store: &StatsigStore,
    eval_func: impl Fn(&APISpec) -> EvalResult,
) {
    result.insert(
        "explicit_parameters".into(),
        json!(spec.explicit_parameters.as_ref().unwrap_or(&vec![])),
    );
    result.insert("undelegated_secondary_exposures".into(), json!([]));

    let delegate = unwrap_or_noop!(&eval_result.config_delegate);
    if delegate.is_empty() {
        return;
    }

    let local_result = spec_store.use_spec("config", delegate.as_str(), |delegate_spec, _| {
        let mut local_result: HashMap<String, Value> = HashMap::new();
        let delegate_spec = unwrap_or_return!(delegate_spec, local_result);
        let delegate_result = eval_func(delegate_spec);

        local_result.insert(
            "allocated_experiment_name".into(),
            json!(hash_name(delegate)),
        );
        local_result.insert(
            "is_user_in_experiment".into(),
            json!(delegate_result.is_experiment_group),
        );
        local_result.insert(
            "is_experiment_active".into(),
            json!(delegate_spec.is_active),
        );
        local_result.insert(
            "explicit_parameters".into(),
            json!(delegate_spec
                .explicit_parameters
                .as_ref()
                .unwrap_or(&vec![])),
        );
        local_result.insert(
            "undelegated_secondary_exposures".into(),
            json!(clean_exposures(
                &delegate_result.undelegated_secondary_exposures
            )),
        );

        local_result
    });

    merge_hash_value(result, local_result);
}

fn merge_json_value(left: &Value, right: Value) -> Value {
    if let (Value::Object(left), Value::Object(right)) = (left, right) {
        let mut result = left.clone();
        for (key, value) in right.iter() {
            result.insert(key.clone(), value.clone());
        }

        return json!(result);
    }

    left.clone()
}

fn merge_hash_value(left: &mut HashMap<String, Value>, right: HashMap<String, Value>) {
    for (key, value) in right.iter() {
        left.insert(key.clone(), value.clone());
    }
}
