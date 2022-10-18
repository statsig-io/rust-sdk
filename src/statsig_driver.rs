use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, RwLock};
use serde::__private::de::Borrowed;
use crate::helpers::make_arc;
use crate::statsig_evaluator::StatsigEvaluator;
use crate::statsig_network::StatsigNetwork;
use crate::statsig_options::StatsigOptions;
use crate::statsig_user::StatsigUser;
use crate::statsig_store::StatsigStore;

pub struct StatsigDriver {
    secret: String,
    options: StatsigOptions,
    store: Arc<Mutex<StatsigStore>>,
    evaluator: Arc<Mutex<StatsigEvaluator>>,
}

impl StatsigDriver {
    pub fn new(secret: &str, options: StatsigOptions) -> Self {
        let network = make_arc(StatsigNetwork::new());
        let store = make_arc(StatsigStore::new(network.clone()));
        let evaluator = make_arc(StatsigEvaluator::new(store.clone()));

        return StatsigDriver {
            secret: String::from(secret),
            options,
            store,
            evaluator,
        };
    }

    pub async fn initialize(&mut self) {
        self.store.lock().unwrap().download_config_specs().await;
    }

    pub async fn check_gate(&mut self, user: &StatsigUser, gate_name: &String) -> bool {
        self.evaluator.lock().unwrap().check_gate(user, gate_name).await;
        return false;
    }
}
