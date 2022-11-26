use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

use tokio::runtime::Handle;
use tokio::task::JoinHandle;

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

    pub fn use_spec<T>(&self, spec_type: &str, spec_name: &String, func: impl Fn(Option<&APISpec>) -> T) -> T
    {
        let specs = self.specs.read().ok().unwrap();
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
                    sleep(interval);
                    Self::download_config_specs_impl(&network, &specs).await;
                };
            })
        );
    }

    async fn download_config_specs_impl(network: &StatsigNetwork, specs: &RwLock<Specs>) -> Option<()> {
        let downloaded_configs = network.download_config_specs().await?;

        if !downloaded_configs.has_updates {
            return None;
        }

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
            mut_specs.update(new_specs);
        };

        None
    }
}

struct Specs {
    gates: HashMap<String, APISpec>,
    configs: HashMap<String, APISpec>,
    layers: HashMap<String, APISpec>,
}

impl Specs {
    pub fn new() -> Specs {
        Specs {
            gates: HashMap::new(),
            configs: HashMap::new(),
            layers: HashMap::new(),
        }
    }

    pub fn update(&mut self, new_specs: Specs) {
        self.gates = new_specs.gates;
        self.configs = new_specs.configs;
        self.layers = new_specs.layers;
    }
}
