use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::runtime::Handle;

use crate::statsig::internal::data_types::APIDownloadedConfigsResponse::WithUpdates;
use crate::statsig::internal::evaluation::specs::Specs;
use crate::statsig::statsig_datastore;
use crate::{StatsigDatastore, StatsigOptions};
use statsig_datastore::CONFIG_SPEC_KEY;

use super::data_types::{APIDownloadedConfigsResponse, APIDownloadedConfigsWithUpdates, APISpec};
use super::statsig_network::StatsigNetwork;

pub struct StatsigStore {
    pub specs: Arc<RwLock<Specs>>,

    runtime_handle: Handle,
    network: Arc<StatsigNetwork>,
    datastore: Option<Arc<dyn StatsigDatastore>>,
    sync_interval_ms: u32,
}

impl StatsigStore {
    pub fn new(
        runtime_handle: &Handle,
        network: Arc<StatsigNetwork>,
        options: &StatsigOptions,
    ) -> Self {
        StatsigStore {
            runtime_handle: runtime_handle.clone(),
            network,
            datastore: options.datastore.clone(),
            specs: Arc::from(RwLock::from(Specs::new())),
            sync_interval_ms: options.rulesets_sync_interval_ms,
        }
    }

    pub async fn initialize(&self) {
        if let Some(store) = &self.datastore {
            store.initialize().await;
        }
        self.initialize_config_specs().await;
        self.spawn_bg_thread();
    }

    pub fn shutdown(&self) {
        if let Some(store) = &self.datastore {
            store.shutdown();
        }
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

    async fn initialize_config_specs(&self) {
        let mut response = None;
        if let Some(store) = &self.datastore {
            response = Self::fetch_and_process_configs_from_datstore(&**store, &self.specs).await;
        }
        if response.is_none() {
            Self::fetch_and_process_configs_from_network(
                &self.network,
                &self.datastore,
                &self.specs,
            )
            .await;
        }
    }

    fn spawn_bg_thread(&self) {
        let network = self.network.clone();
        let datastore = self.datastore.clone();
        let specs = self.specs.clone();
        let interval = Duration::from_millis(self.sync_interval_ms as u64);

        self.runtime_handle.spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                match &datastore {
                    Some(store) if store.should_be_used_for_querying_updates() => {
                        Self::fetch_and_process_configs_from_datstore(&**store, &specs).await;
                    }
                    _ => {
                        Self::fetch_and_process_configs_from_network(&network, &datastore, &specs)
                            .await;
                    }
                };
            }
        });
    }

    async fn save_config_specs_to_datastore(datastore: &Option<Arc<dyn StatsigDatastore>>, specs: &str) {
        if let Some(store) = datastore {
            store.set(CONFIG_SPEC_KEY, specs).await;
        }
    }

    async fn fetch_config_specs_from_network(
        network: &StatsigNetwork,
        specs: &RwLock<Specs>,
    ) -> Option<String> {
        let last_sync_time = match specs.read().ok() {
            Some(t) => t.last_sync_time,
            _ => 0,
        };

        network.download_config_specs(last_sync_time).await
    }

    async fn fetch_config_specs_from_datastore(datastore: &dyn StatsigDatastore) -> Option<String> {
        datastore.get(CONFIG_SPEC_KEY).await
    }

    async fn fetch_and_process_configs_from_network(
        network: &StatsigNetwork,
        datastore: &Option<Arc<dyn StatsigDatastore>>,
        specs: &RwLock<Specs>,
    ) -> Option<()> {
        let response = Self::fetch_config_specs_from_network(network, specs).await;
        let configs = match response {
            Some(ref data) => Self::parse_config_specs(data),
            None => {
                println!("[Statsig] No result returned from download_config_specs");
                return None;
            }
        };
        if let Some(WithUpdates(r)) = configs { 
            let valid_update = Self::set_downloaded_config_specs(specs, r.clone());
            match valid_update {
                Some(()) => {
                    let specs_json = serde_json::to_string(&r);
                    if let Ok(specs_string) = specs_json {
                        Self::save_config_specs_to_datastore(datastore, &specs_string).await;
                    }
                    return Some(())
                }
                None => {
                    return None
                }
            }
        }
        None
    }

    async fn fetch_and_process_configs_from_datstore(
        datastore: &dyn StatsigDatastore,
        specs: &RwLock<Specs>,
    ) -> Option<()> {
        let response = Self::fetch_config_specs_from_datastore(datastore).await?;
        let configs = Self::parse_config_specs(&response);
        if let Some(WithUpdates(r)) = configs {
            Self::set_downloaded_config_specs(specs, r);
            return Some(());
        }
        None
    }

    fn set_downloaded_config_specs(
        specs: &RwLock<Specs>,
        downloaded_configs: APIDownloadedConfigsWithUpdates,
    ) -> Option<()> {
        let last_sync_time = match specs.read().ok() {
            Some(t) => t.last_sync_time,
            _ => 0,
        };
        if downloaded_configs.time <= last_sync_time {
            return None;
        }
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
            new_specs.last_sync_time = downloaded_configs.time;
            mut_specs.update(new_specs);
        };
        return Some(())
    }

    fn parse_config_specs(text: &str) -> Option<APIDownloadedConfigsResponse> {
        serde_json::from_str::<APIDownloadedConfigsResponse>(text).ok()
    }
}
