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
    store: Arc<StatsigStore>,
    evaluator: Arc<StatsigEvaluator>,
}

impl StatsigDriver {
    pub fn new(secret_key: &str, options: StatsigOptions) -> Self {
        let network = Arc::from(StatsigNetwork::new(secret_key, &options));
        let store = Arc::from(StatsigStore::new(network.clone()));
        let evaluator = Arc::from(StatsigEvaluator::new(store.clone()));

        return StatsigDriver {
            secret_key: secret_key.to_string(),
            options,
            store,
            evaluator,
        };
    }

    pub async fn initialize(&self) -> Option<()> {
        self.store.download_config_specs().await
    }

    pub async fn check_gate(&self, user: &StatsigUser, gate_name: &String) -> bool {
        let spec_eval = self.evaluator.check_gate(user, gate_name);
        return spec_eval.bool_value;
    }
}
