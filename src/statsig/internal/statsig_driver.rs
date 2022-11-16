use std::collections::HashMap;
use std::sync::Arc;

use serde_json::from_value;

use crate::{StatsigEvent, StatsigOptions};
use crate::statsig::dynamic_config::DynamicConfig;
use crate::statsig::internal::statsig_event_internal::make_config_exposure;
use crate::StatsigUser;

use super::evaluation::StatsigEvaluator;
use super::statsig_event_internal::{finalize_event, make_gate_exposure};
use super::statsig_logger::StatsigLogger;
use super::statsig_network::StatsigNetwork;
use super::statsig_store::StatsigStore;

pub struct StatsigDriver {
    pub secret_key: String,
    pub options: StatsigOptions,
    store: Arc<StatsigStore>,
    evaluator: StatsigEvaluator,
    logger: StatsigLogger,
}

impl StatsigDriver {
    pub fn new(secret_key: &str, options: StatsigOptions) -> Self {
        let network = Arc::from(StatsigNetwork::new(secret_key, &options));
        let store = Arc::from(StatsigStore::new(network.clone()));
        let evaluator = StatsigEvaluator::new(store.clone());
        let logger = StatsigLogger::new(network.clone(), &options);

        return StatsigDriver {
            secret_key: secret_key.to_string(),
            options,
            store,
            evaluator,
            logger,
        };
    }

    pub async fn initialize(&self) -> Option<()> {
        self.store.download_config_specs().await
    }

    pub async fn shutdown(&self) {
        self.logger.flush().await;
    }

    pub fn check_gate(&self, user: StatsigUser, gate_name: &String) -> bool {
        let eval_result = self.evaluator.check_gate(&user, gate_name);
        
        self.logger.enqueue(make_gate_exposure(
            user, gate_name, &eval_result, &self.options.environment,
        ));
        
        return eval_result.bool_value;
    }

    pub fn get_config(&self, user: StatsigUser, config_name: &String) -> DynamicConfig {
        let eval_result = self.evaluator.get_config(&user, config_name);

        self.logger.enqueue(make_config_exposure(
            user, config_name, &eval_result, &self.options.environment,
        ));

        let mut value = HashMap::from([]);
        if let Some(json_value) = eval_result.json_value {
            if let Ok(deserialized) = from_value(json_value) {
                value = deserialized;
            }
        }

        return DynamicConfig { name: config_name.clone(), value, rule_id: eval_result.rule_id };
    }

    pub fn log_event(&self, event: StatsigEvent) {
        self.logger.enqueue(finalize_event(
            event,
            &self.options.environment,
        ))
    }
}
