use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use crate::statsig::internal::data_types::APIDownloadedConfigsResponse::{NoUpdates, WithUpdates};
use crate::statsig::internal::evaluation::specs::Specs;
use crate::StatsigOptions;

use super::data_types::APISpec;
use super::statsig_network::StatsigNetwork;

pub struct StatsigStore {
    pub specs: Arc<RwLock<Specs>>,

    runtime_handle: Handle,
    network: Arc<StatsigNetwork>,
    sync_interval_ms: u32,
    bg_thread_handle: Option<JoinHandle<()>>,
}

impl StatsigStore {
    pub fn new(
        runtime_handle: &Handle,
        network: Arc<StatsigNetwork>,
        options: &StatsigOptions,
    ) -> Self {
        let mut inst = StatsigStore {
            runtime_handle: runtime_handle.clone(),
            network,
            specs: Arc::from(RwLock::from(Specs::new())),
            sync_interval_ms: options.rulesets_sync_interval_ms,
            bg_thread_handle: None,
        };
        inst.spawn_bg_thread();
        inst
    }

    pub async fn download_config_specs(&self) -> Option<()> {
        Self::download_config_specs_impl(&self.network, &self.specs).await
    }

    pub fn use_spec<T>(
        &self,
        spec_type: &str,
        spec_name: &str,
        func: impl Fn(Option<&APISpec>) -> T,
    ) -> T {
        let specs = self.specs.read().expect("Specs read lock");
        let specs_map = match spec_type {
            "config" => &specs.configs,
            "layer" => &specs.layers,
            _ => &specs.gates,
        };

        func(specs_map.get(spec_name))
    }

    pub fn get_layer_name_for_experiment(&self, experiment_name: &String) -> Option<String> {
        let specs = self.specs.read().ok()?;
        return specs.experiment_to_layer.get(experiment_name).cloned();
    }

    fn spawn_bg_thread(&mut self) {
        let network = self.network.clone();
        let specs = self.specs.clone();
        let interval = Duration::from_millis(self.sync_interval_ms as u64);

        self.bg_thread_handle = Some(self.runtime_handle.spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                Self::download_config_specs_impl(&network, &specs).await;
            }
        }));
    }

    async fn download_config_specs_impl(
        network: &StatsigNetwork,
        specs: &RwLock<Specs>,
    ) -> Option<()> {
        let last_sync_time = match specs.read().ok() {
            Some(t) => t.last_sync_time,
            _ => 0,
        };

        let downloaded_configs = match network.download_config_specs(last_sync_time).await {
            Some(WithUpdates(r)) => r,
            Some(NoUpdates(..)) => return None,
            None => {
                println!("[Statsig] No result returned from download_config_specs");
                return None;
            }
        };

        let mut new_specs = Specs::new();
        for feature_gate in downloaded_configs.feature_gates {
            new_specs
                .gates
                .insert(feature_gate.name.to_string(), feature_gate);
        }

        for dynamic_config in downloaded_configs.dynamic_configs {
            new_specs
                .configs
                .insert(dynamic_config.name.to_string(), dynamic_config);
        }

        for layer_config in downloaded_configs.layer_configs {
            new_specs
                .layers
                .insert(layer_config.name.to_string(), layer_config);
        }

        for (layer_name, experiments) in downloaded_configs.layers.unwrap_or_default().iter() {
            for experiment_name in experiments {
                new_specs
                    .experiment_to_layer
                    .insert(experiment_name.clone(), layer_name.clone());
            }
        }

        if let Ok(mut mut_specs) = specs.write() {
            new_specs.last_sync_time = downloaded_configs
                .time
                .as_u64()
                .unwrap_or(mut_specs.last_sync_time);
            mut_specs.update(new_specs);
        };

        None
    }
}
