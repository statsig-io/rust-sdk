use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::data_types::{APIDownloadedConfigs, APISpec};
use super::statsig_network::StatsigNetwork;

pub struct StatsigStore {
    specs: RwLock<Specs>,
    network: Arc<StatsigNetwork>,
}

impl StatsigStore {
    pub fn new(network: Arc<StatsigNetwork>) -> StatsigStore {
        StatsigStore { network, specs: RwLock::from(Specs::new()) }
    }

    pub async fn download_config_specs(&self) -> Option<()> {
        let result = self.network.download_config_specs().await?;
        Some(self.parse_specs(result))
    }

    fn parse_specs(&self, downloaded_configs: APIDownloadedConfigs) {
        if !downloaded_configs.has_updates {
            return;
        }

        let mut specs = Specs::new();
        for feature_gate in downloaded_configs.feature_gates {
            specs.gates.insert(feature_gate.name.to_string(), feature_gate);
        }

        for dynamic_config in downloaded_configs.dynamic_configs {
            specs.configs.insert(dynamic_config.name.to_string(), dynamic_config);
        }

        for layer_config in downloaded_configs.layer_configs {
            specs.layers.insert(layer_config.name.to_string(), layer_config);
        }

        if let Some(mut mut_specs) = self.specs.write().ok() {
            mut_specs.update(specs);
        };
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
