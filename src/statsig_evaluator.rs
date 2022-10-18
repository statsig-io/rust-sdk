use std::borrow::Borrow;
use std::cell::RefCell;
use std::iter::Map;
use std::ops::Deref;
use std::rc::Weak;
use std::sync::{Arc, Mutex, RwLock};
use serde_json::Value;
use crate::data_types::APIConfig;
use crate::statsig_store::StatsigStore;
use crate::StatsigUser;

pub struct StatsigEvaluator {
    pub spec_store: Arc<Mutex<StatsigStore>>,
}

pub struct ConfigEvaluation {
    name: String,
    gate_value: bool,
    json_value: Option<Map<String, Value>>,
    rule_id: String
}

impl ConfigEvaluation {
    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
            gate_value: false,
            json_value: None,
            rule_id: String::from("")
        }
    }
    
    pub fn with_rule_id(mut self, rule_id: &string) -> Self {
        self.rule_id = rule_id.clone();
        return self;
    }
}

impl StatsigEvaluator {
    pub fn new(spec_store: Arc<Mutex<StatsigStore>>) -> StatsigEvaluator {
        StatsigEvaluator {
            spec_store
        }
    }

    pub async fn check_gate(&mut self, user: &StatsigUser, gate_name: &String) -> ConfigEvaluation {
        let mut store = self.spec_store.lock().unwrap();
        let gate = store.get_gate(gate_name);
        match gate {
            Some(config) => self.eval_config(user, config),
            None => ConfigEvaluation::new(gate_name)
        }
    }
    
    fn eval_config(&self, user: &StatsigUser, config: &APIConfig) -> ConfigEvaluation {
        if !config.enabled {
            return ConfigEvaluation::new(name);
        }
        
        ConfigEvaluation {
            name: config.name.clone(),
            gate_value: false,
        }
    }
}
