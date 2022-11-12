use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, RwLock};
use serde::__private::de::Borrowed;

use crate::statsig::helpers::make_arc;
use crate::statsig::statsig_evaluator::StatsigEvaluator;
use crate::statsig::statsig_network::StatsigNetwork;
use crate::statsig::statsig_options::StatsigOptions;
use crate::statsig::statsig_store::StatsigStore;
use crate::StatsigUser;

pub struct StatsigDriver {
    secret_key: String,
    options: StatsigOptions,
    store: Arc<Mutex<StatsigStore>>,
    evaluator: Arc<Mutex<StatsigEvaluator>>,
}

impl StatsigDriver {
    pub fn new(secret_key: &str, options: StatsigOptions) -> Self {
        let network = make_arc(StatsigNetwork::new(secret_key, &options));
        let store = make_arc(StatsigStore::new(network.clone()));
        let evaluator = make_arc(StatsigEvaluator::new(store.clone()));

        return StatsigDriver {
            secret_key: secret_key.to_string(),
            options,
            store,
            evaluator,
        };
    }

    pub async fn initialize(&mut self) {
        self.store.lock().unwrap().download_config_specs().await;
    }

    pub async fn check_gate(&mut self, user: &StatsigUser, gate_name: &String) -> bool {
        let spec_eval = self.evaluator.lock().unwrap().check_gate(user, gate_name).await;
        return spec_eval.bool_value;
    }
}
