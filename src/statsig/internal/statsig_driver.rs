use std::sync::Arc;

use crate::{StatsigEvent, StatsigOptions};
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

    pub fn log_event(&self, event: StatsigEvent) {
        self.logger.enqueue(finalize_event(
            event,
            &self.options.environment,
        ))
    }
}
