use std::collections::HashMap;
use serde_json::{Value, from_value};
use serde::de::DeserializeOwned;
use crate::statsig::internal::EvalResult;
use crate::{Statsig, StatsigUser};

pub struct Layer {
    pub name: String,
    pub rule_id: String,
    
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

        return default;
    }
}

pub struct  LayerLogData {
    pub(crate) eval_result: EvalResult,
    pub(crate) user: StatsigUser,
}

