use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::data_types::{APIDownloadedConfigs, APISpec};
use super::network::StatsigNetwork;

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
            // specs.gates.insert(feature_gate.name.to_string(), Arc::from(feature_gate));
            specs.gates.insert(feature_gate.name.to_string(), feature_gate);
        }

        for dynamic_config in downloaded_configs.dynamic_configs {
            specs.configs.insert(dynamic_config.name.to_string(), Arc::from(dynamic_config));
        }

        for layer_config in downloaded_configs.layer_configs {
            specs.layers.insert(layer_config.name.to_string(), Arc::from(layer_config));
        }

        if let Some(mut mut_specs) = self.specs.write().ok() {
            mut_specs.update(specs);
        };
    }

    // pub fn get_gate(&self, gate_name: &String) -> Option<Arc<APISpec>> {
    //     match self.specs.lock().ok()?.gates.get(gate_name.as_str()) {
    //         Some(spec) => Some(spec.clone()),
    //         None => None
    //     }
    // }

    pub fn use_gate<T>(&self, gate_name: &String, func: impl Fn(Option<&APISpec>) -> T) -> T
    {
        func(self.specs.read().ok().unwrap().gates.get(gate_name))
    }
}

struct Specs {
    gates: HashMap<String, APISpec>,
    configs: HashMap<String, Arc<APISpec>>,
    layers: HashMap<String, Arc<APISpec>>,
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
