use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{from_value, Value};
use tokio::runtime::{Builder, Runtime};

use crate::statsig::internal::statsig_event_internal::{make_config_exposure, make_layer_exposure};
use crate::StatsigUser;
use crate::{LayerLogData, StatsigEvent, StatsigOptions};

use super::evaluation::StatsigEvaluator;
use super::statsig_event_internal::{finalize_event, make_gate_exposure};
use super::statsig_logger::StatsigLogger;
use super::statsig_network::StatsigNetwork;
use super::statsig_store::StatsigStore;
use super::DynamicConfig;
use super::Layer;

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
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => return Err(e),
        };

        let network = Arc::from(StatsigNetwork::new(secret_key, &options));
        let logger = StatsigLogger::new(runtime.handle(), network.clone(), &options);
        let store = Arc::from(StatsigStore::new(
            runtime.handle(),
            network.clone(),
            &options,
        ));
        let evaluator = StatsigEvaluator::new(store.clone(), &options);

        Ok(StatsigDriver {
            secret_key: secret_key.to_string(),
            options,
            runtime: Mutex::from(Some(runtime)),
            store,
            evaluator,
            logger,
        })
    }

    pub async fn initialize(&self) {
        // bubble up error?
        self.store.download_config_specs().await;
    }

    pub fn shutdown(&self) {
        self.logger.flush_blocking();

        if let Ok(mut lock) = self.runtime.lock() {
            if let Some(runtime) = lock.take() {
                runtime.shutdown_timeout(Duration::from_secs(10))
            }
        }
    }

    pub fn check_gate(&self, user: &StatsigUser, gate_name: &str) -> bool {
        let normalized_user = &self.get_normalized_user_copy(user);
        let eval_result = self.evaluator.check_gate(normalized_user, gate_name);

        self.logger.enqueue(make_gate_exposure(
            normalized_user,
            gate_name,
            &eval_result,
            &self.options.environment,
        ));

        eval_result.bool_value
    }

    pub fn get_config(&self, user: &StatsigUser, config_name: &str) -> DynamicConfig {
        let normalized_user = &self.get_normalized_user_copy(user);
        let eval_result = self.evaluator.get_config(normalized_user, config_name);

        self.logger.enqueue(make_config_exposure(
            normalized_user,
            config_name,
            &eval_result,
            &self.options.environment,
        ));

        let mut value = HashMap::from([]);
        if let Some(json_value) = eval_result.json_value {
            if let Ok(deserialized) = from_value(json_value) {
                value = deserialized;
            }
        }

        DynamicConfig {
            name: config_name.to_string(),
            value,
            rule_id: eval_result.rule_id,
        }
    }

    pub fn get_layer(&self, user: &StatsigUser, layer_name: &str) -> Layer {
        let normalized_user = self.get_normalized_user_copy(user);
        let eval_result = self.evaluator.get_layer(&normalized_user, layer_name);

        let mut value = HashMap::from([]);
        if let Some(ref json_value) = eval_result.json_value {
            if let Ok(deserialized) = from_value(json_value.clone()) {
                value = deserialized;
            }
        }

        Layer {
            name: layer_name.to_string(),
            value,
            rule_id: eval_result.rule_id.clone(),
            log_data: LayerLogData {
                user: normalized_user,
                eval_result,
            },
        }
    }

    pub fn log_event(&self, user: &StatsigUser, event: StatsigEvent) {
        self.logger
            .enqueue(finalize_event(user, event, &self.options.environment))
    }

    pub fn get_client_initialize_response(&self, user: &StatsigUser) -> Value {
        let normalized_user = self.get_normalized_user_copy(user);
        self.evaluator
            .get_client_initialize_response(&normalized_user)
    }

    pub(crate) fn log_layer_parameter_exposure(
        &self,
        layer: &Layer,
        parameter_name: &str,
        log_data: &LayerLogData,
    ) {
        self.logger.enqueue(make_layer_exposure(
            &log_data.user,
            &layer.name,
            parameter_name,
            &log_data.eval_result,
            &self.options.environment,
        ));
    }

    fn get_normalized_user_copy(&self, user: &StatsigUser) -> StatsigUser {
        let mut normalized_user = user.clone();
        if self.options.environment.is_some() {
            normalized_user.statsig_environment = self.options.environment.clone();
        }
        normalized_user
    }

    #[doc(hidden)]
    #[cfg(statsig_kong)]
    pub fn __unsafe_shutdown(&self) {
        if let Some(mut lock) = self.runtime.lock().ok() {
            if let Some(runtime) = lock.take() {
                runtime.shutdown_timeout(Duration::from_secs(10))
            }
        }
    }
}
