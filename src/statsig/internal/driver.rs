use std::sync::{Arc, Mutex};

use crate::StatsigOptions;
use crate::StatsigUser;

use super::evaluation::evaluator::StatsigEvaluator;
use super::network::StatsigNetwork;
use super::store::StatsigStore;
use super::helpers::make_arc;

pub struct StatsigDriver {
    pub secret_key: String,
    pub options: StatsigOptions,
    store: Arc<Mutex<StatsigStore>>,
    evaluator: Arc<Mutex<StatsigEvaluator>>,
}

impl StatsigDriver {
    pub fn new(secret_key: &str, options: StatsigOptions) -> Self {
        let network = make_arc(StatsigNetwork::new(secret_key, &options));
        let store = make_arc(StatsigStore::new(network.clone()));
        let evaluator = make_arc(StatsigEvaluator::new(store.clone()));

        return StatsigDriver {
            secret_key: secret_key.to_string(),
            options,
            store,
            evaluator,
        };
    }

    pub async fn initialize(&mut self) {
        self.store.lock().unwrap().download_config_specs().await;
    }

    pub async fn check_gate(&mut self, user: &StatsigUser, gate_name: &String) -> bool {
        let spec_eval = self.evaluator.lock().unwrap().check_gate(user, gate_name).await;
        return spec_eval.bool_value;
    }
}
