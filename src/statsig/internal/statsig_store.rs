use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::runtime::Handle;
use tokio::task::JoinHandle;
use crate::statsig::internal::data_types::APIDownloadedConfigsResponse::{NoUpdates, WithUpdates};

use crate::StatsigOptions;

use super::data_types::APISpec;
use super::statsig_network::StatsigNetwork;

pub struct StatsigStore {
    runtime_handle: Handle,
    specs: Arc<RwLock<Specs>>,
    network: Arc<StatsigNetwork>,
    sync_interval_ms: u32,
    bg_thread_handle: Option<JoinHandle<()>>,
}

impl StatsigStore {
    pub fn new(runtime_handle: &Handle, network: Arc<StatsigNetwork>, options: &StatsigOptions) -> Self {
        let mut inst = StatsigStore {
            runtime_handle: runtime_handle.clone(),
            network,
            specs: Arc::from(RwLock::from(Specs::new())),
            sync_interval_ms: options.rulesets_sync_interval_ms,
            bg_thread_handle: None,
        };
        inst.spawn_bg_thread();
        return inst;
    }

    pub async fn download_config_specs(&self) -> Option<()> {
        Self::download_config_specs_impl(&self.network, &self.specs).await
    }

    pub fn use_spec<T>(&self, spec_type: &str, spec_name: &str, func: impl Fn(Option<&APISpec>) -> T) -> T
    {
        let specs = self.specs.read().ok().expect("Specs read lock");
        let specs_map = match spec_type {
            "config" => &specs.configs,
            "layer" => &specs.layers,
            _ => &specs.gates,
        };

        func(specs_map.get(spec_name))
    }

    fn spawn_bg_thread(&mut self) {
        let network = self.network.clone();
        let specs = self.specs.clone();
        let interval = Duration::from_millis(self.sync_interval_ms as u64);

        self.bg_thread_handle = Some(
            self.runtime_handle.spawn(async move {
                loop {
                    tokio::time::sleep(interval).await;
                    Self::download_config_specs_impl(&network, &specs).await;
                };
            })
        );
    }

    async fn download_config_specs_impl(network: &StatsigNetwork, specs: &RwLock<Specs>) -> Option<()> {
        let last_sync_time = match specs.read().ok() {
            Some(t) => t.last_sync_time,
            _ => 0
        };
        
        let downloaded_configs = match network.download_config_specs(last_sync_time).await? { 
            WithUpdates(r) => r,
            NoUpdates(..) => return None
        };

        let mut new_specs = Specs::new();
        for feature_gate in downloaded_configs.feature_gates {
            new_specs.gates.insert(feature_gate.name.to_string(), feature_gate);
        }

        for dynamic_config in downloaded_configs.dynamic_configs {
            new_specs.configs.insert(dynamic_config.name.to_string(), dynamic_config);
        }

        for layer_config in downloaded_configs.layer_configs {
            new_specs.layers.insert(layer_config.name.to_string(), layer_config);
        }

        if let Some(mut mut_specs) = specs.write().ok() {
            new_specs.last_sync_time = downloaded_configs.time.as_u64()
                .unwrap_or(mut_specs.last_sync_time);
            mut_specs.update(new_specs);
        };

        None
    }
}

struct Specs {
    last_sync_time: u64,
    gates: HashMap<String, APISpec>,
    configs: HashMap<String, APISpec>,
    layers: HashMap<String, APISpec>,
}

impl Specs {
    pub fn new() -> Specs {
        Specs {
            last_sync_time: 0,
            gates: HashMap::new(),
            configs: HashMap::new(),
            layers: HashMap::new(),
        }
    }

    pub fn update(&mut self, new_specs: Specs) {
        self.last_sync_time = new_specs.last_sync_time;
        self.gates = new_specs.gates;
        self.configs = new_specs.configs;
        self.layers = new_specs.layers;
    }
}
