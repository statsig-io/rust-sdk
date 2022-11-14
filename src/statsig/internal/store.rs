use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::data_types::{APIDownloadedConfigs, APISpec};
use super::network::StatsigNetwork;

pub struct StatsigStore {
    specs: Specs,
    network: Arc<Mutex<StatsigNetwork>>,
}

impl StatsigStore {
    pub fn new(network: Arc<Mutex<StatsigNetwork>>) -> StatsigStore {
        StatsigStore { network, specs: Specs::new() }
    }

    pub async fn download_config_specs(&mut self) -> Option<()> {
        let result = self.network.lock().ok()?.download_config_specs().await?;
        Some(self.parse_specs(result))
    }

    fn parse_specs(&mut self, downloaded_configs: APIDownloadedConfigs) {
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

        self.specs = specs;
    }

    pub fn get_gate(&self, gate_name: &String) -> Option<&APISpec> {
        return self.specs.gates.get(gate_name.as_str());
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
}
