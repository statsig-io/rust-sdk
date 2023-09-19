use crate::statsig::internal::data_types::APISpec;
use std::collections::HashMap;

pub struct Specs {
    pub last_sync_time: u64,
    pub gates: HashMap<String, APISpec>,
    pub configs: HashMap<String, APISpec>,
    pub layers: HashMap<String, APISpec>,
    pub experiment_to_layer: HashMap<String, String>,
}

impl Specs {
    pub fn new() -> Specs {
        Specs {
            last_sync_time: 0,
            gates: HashMap::new(),
            configs: HashMap::new(),
            layers: HashMap::new(),
            experiment_to_layer: HashMap::new(),
        }
    }

    pub fn update(&mut self, new_specs: Specs) {
        self.last_sync_time = new_specs.last_sync_time;
        self.gates = new_specs.gates;
        self.configs = new_specs.configs;
        self.layers = new_specs.layers;
        self.experiment_to_layer = new_specs.experiment_to_layer;
    }
}
