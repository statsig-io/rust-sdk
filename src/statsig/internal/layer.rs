use crate::statsig::internal::EvalResult;
use crate::{Statsig, StatsigUser};
use serde::de::DeserializeOwned;
use serde_json::{from_value, Value};
use std::collections::HashMap;

use super::evaluation::eval_details::EvalDetails;

pub struct Layer {
    pub name: String,
    pub rule_id: String,
    pub evaluation_details: EvalDetails,

    pub(crate) value: HashMap<String, Value>,
    pub(crate) log_data: LayerLogData,
}

impl Layer {
    pub fn get<T: DeserializeOwned>(&self, key: &str, default: T) -> T {
        if !self.value.contains_key(key) {
            return default;
        }

        if let Ok(value) = from_value(self.value[key].clone()) {
            Statsig::log_layer_parameter_exposure(self, key, &self.log_data);
            return value;
        }

        default
    }
}

pub struct LayerLogData {
    pub(crate) eval_result: EvalResult,
    pub(crate) user: StatsigUser,
}
