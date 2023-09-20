use std::collections::HashMap;

use serde_json::Value::Null;
use serde_json::{json, Value};

use crate::statsig::internal::data_types::APISpec;
use crate::statsig::internal::statsig_store::StatsigStore;
use crate::statsig::internal::EvalResult;
use crate::{unwrap_or_return, StatsigUser};

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
                }

                "dynamic_config" => {
                    result.insert("value".into(), json!(eval_result.json_value));
                    result.insert("group".into(), json!(eval_result.rule_id));
                    result.insert(
                        "is_device_based".into(),
                        json!(spec.id_type.to_lowercase() == "stableid"),
                    );

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
                        _ => return None,
                    }
                }

                _ => return None,
            }

            Some(json!(result))
        };

        let eval_all = |spec_type: &str| -> Vec<Option<Value>> {
            let iter = match spec_type {
                "gates" => specs.gates.iter(),
                "configs" => specs.configs.iter(),
                "layers" => specs.layers.iter(),
                _ => return vec![],
            };
            iter.map(|(name, spec)| get_evaluated_spec(name, spec))
                .filter(|result| result.is_some())
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
    name.to_owned()
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
        json!(spec.is_experiment_active.unwrap_or(false)),
    );

    if !spec.has_shared_params.unwrap_or(false) {
        return;
    }

    result.insert("is_in_layer".into(), json!(true));

    let explicit_params = match &spec.explicit_parameters {
        Some(params) => json!(params),
        None => json!([]),
    };
    result.insert("explicit_parameters".into(), explicit_params);

    let layer_name = match spec_store.get_layer_name_for_experiment(&spec.name) {
        Some(layer_name) => layer_name,
        None => return ,
    };
    let layer_value = spec_store.use_spec("layer", layer_name.as_str(), |layer| {
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
    let explicit_params = match &spec.explicit_parameters {
        Some(params) => json!(params),
        None => json!([]),
    };
    result.insert("explicit_parameters".into(), explicit_params);

    if let Some(delegate) = &eval_result.config_delegate {
        if delegate.is_empty() {
            return;
        }

        if let Some((is_active, delegate_result)) =
            spec_store.use_spec("config", delegate.as_str(), |delegate_spec| {
                let delegate_spec = unwrap_or_return!(delegate_spec, None);
                let is_active = unwrap_or_return!(delegate_spec.is_active, None);
                Some((is_active, eval_func(delegate_spec)))
            })
        {
            result.insert(
                "allocated_experiment_name".into(),
                json!(hash_name(delegate)),
            );
            result.insert(
                "is_user_in_experiment".into(),
                json!(delegate_result.is_experiment_group),
            );
            result.insert("is_experiment_active".into(), json!(is_active));
            result.insert(
                "explicit_parameters".into(),
                json!(delegate_result.explicit_parameters),
            );
        }
    }
}

fn merge_json_value(left: &Value, right: Value) -> Value {
    if let (Value::Object(left), Value::Object(right)) = (left, right) {
        let mut base = left.clone();
        for (key, value) in right.iter() {
            base.insert(key.clone(), value.clone());
        }
    }

    left.clone()
}
