use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::from_value;
use tokio::runtime::{Builder, Runtime};

use crate::{StatsigEvent, StatsigOptions};
use crate::statsig::internal::statsig_event_internal::make_config_exposure;
use crate::StatsigUser;

use super::DynamicConfig;
use super::evaluation::StatsigEvaluator;
use super::Layer;
use super::statsig_event_internal::{finalize_event, make_gate_exposure};
use super::statsig_logger::StatsigLogger;
use super::statsig_network::StatsigNetwork;
use super::statsig_store::StatsigStore;

pub struct StatsigDriver {
    pub secret_key: String,
    pub options: StatsigOptions,
    runtime: Mutex<Option<Runtime>>,
    store: Arc<StatsigStore>,
    evaluator: StatsigEvaluator,
    logger: StatsigLogger,
}

impl StatsigDriver {
    pub fn new(secret_key: &str, options: StatsigOptions) -> std::io::Result<Self> {
        let runtime = match Builder::new_multi_thread()
            .worker_threads(3)
            .thread_name("statsig")
            .build() {
            Ok(rt) => rt,
            Err(e) => return Err(e)
        };

        let network = Arc::from(StatsigNetwork::new(secret_key, &options));
        let store = Arc::from(StatsigStore::new(runtime.handle(), network.clone()));
        let evaluator = StatsigEvaluator::new(store.clone());
        let logger = StatsigLogger::new(runtime.handle(), network.clone(), &options);

        return Ok(
            StatsigDriver {
                secret_key: secret_key.to_string(),
                options,
                runtime: Mutex::from(Some(runtime)),
                store,
                evaluator,
                logger,
            }
        );
    }

    pub async fn initialize(&self) {
        // bubble up error?
        self.store.download_config_specs().await;
    }

    pub async fn shutdown(&self) {
        self.logger.flush().await;

        if let Some(mut lock) = self.runtime.lock().ok() {
            if let Some(runtime) = lock.take() {
                runtime.shutdown_background()
            }
        }
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

    pub fn get_layer(&self, user: StatsigUser, layer_name: &String) -> Layer {
        let eval_result = self.evaluator.get_config(&user, layer_name);

        // self.logger.enqueue(make_config_exposure(
        //     user, config_name, &eval_result, &self.options.environment,
        // ));

        let mut value = HashMap::from([]);
        if let Some(json_value) = eval_result.json_value {
            if let Ok(deserialized) = from_value(json_value) {
                value = deserialized;
            }
        }

        return Layer { name: layer_name.clone(), value, rule_id: eval_result.rule_id };
    }

    pub fn log_event(&self, event: StatsigEvent) {
        self.logger.enqueue(finalize_event(
            event,
            &self.options.environment,
        ))
    }
}
