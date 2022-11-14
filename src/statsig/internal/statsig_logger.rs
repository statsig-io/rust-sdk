use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use crate::statsig::internal::statsig_network::StatsigNetwork;
use crate::{StatsigEvent, StatsigOptions};

pub struct StatsigLogger {
    network: Arc<StatsigNetwork>,
    events: Arc<RwLock<Vec<StatsigEvent>>>,
    max_queue_size: u32,
    flush_interval_ms: u32,
    bg_thread_handle: Option<JoinHandle<()>>,
}

impl StatsigLogger {
    pub fn new(network: Arc<StatsigNetwork>, options: &StatsigOptions) -> Self {
        let mut inst = Self {
            network,
            events: Arc::from(RwLock::from(vec![])),
            max_queue_size: options.logger_max_queue_size,
            flush_interval_ms: options.logger_flush_interval_ms,
            bg_thread_handle: None,
        };
        inst.spawn_bg_thread();
        inst
    }

    pub fn spawn_bg_thread(&mut self) {
        let events = self.events.clone();
        self.bg_thread_handle = Some(thread::spawn(move || {
            events.write().ok().unwrap().clear();
        }));
    }

    pub fn enqueue(&self, event: StatsigEvent) {
        if let Some(mut mut_events) = self.events.write().ok() {
            mut_events.push(event);
        };
    }
}