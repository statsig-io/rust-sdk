use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio::runtime::Handle;
use tokio::task::JoinHandle;

use crate::statsig::internal::statsig_network::StatsigNetwork;
use crate::StatsigOptions;

use super::statsig_event_internal::StatsigEventInternal;

pub struct StatsigLogger {
    runtime_handle: Handle,
    network: Arc<StatsigNetwork>,
    events: Arc<RwLock<Vec<StatsigEventInternal>>>,
    max_queue_size: usize,
    flush_interval_ms: u32,
    bg_thread_handle: Option<JoinHandle<()>>,
    running_jobs: Arc<RwLock<Vec<JoinHandle<()>>>>,
    is_shutdown: Arc<AtomicBool>,
}

impl StatsigLogger {
    pub fn new(
        runtime_handle: &Handle,
        network: Arc<StatsigNetwork>,
        options: &StatsigOptions,
    ) -> Self {
        let mut inst = Self {
            runtime_handle: runtime_handle.clone(),
            network,
            events: Arc::from(RwLock::from(vec![])),
            max_queue_size: options.logger_max_queue_size as usize,
            flush_interval_ms: options.logger_flush_interval_ms,
            running_jobs: Arc::from(RwLock::from(vec![])),
            bg_thread_handle: None,
            is_shutdown: Arc::new(AtomicBool::new(false)),
        };
        inst.spawn_bg_thread();
        inst
    }

    pub fn enqueue(&self, event: StatsigEventInternal) {
        let mut should_flush = false;
        if let Ok(mut mut_events) = self.events.write() {
            mut_events.push(event);
            should_flush = mut_events.len() > self.max_queue_size;
        };

        if should_flush {
            self.flush();
        }
    }

    pub fn flush(&self) {
        let events = self.events.clone();
        let network = self.network.clone();

        if let Ok(mut lock) = self.running_jobs.write() {
            // Clear any finished jobs
            lock.retain(|x| !x.is_finished());

            lock.push(
                self.runtime_handle
                    .spawn(async move { Self::flush_impl(&network, &events).await }),
            );
        }
    }

    pub fn shutdown(&self) {
        self.is_shutdown.store(true, Ordering::Relaxed);
        let events = self.events.clone();
        let network = self.network.clone();

        #[allow(clippy::await_holding_lock)]
        self.runtime_handle.block_on(async move {
            // Purposely hold onto the running jobs lock while waiting for
            // all job handles to complete to ensure no new job handles
            // are added in parallel
            if let Ok(mut t) = self.running_jobs.clone().write() {
                for handle in t.iter_mut() {
                    let _ = handle.await;
                }
            }
            Self::flush_impl(&network, &events).await;
        });
    }

    async fn flush_impl(network: &StatsigNetwork, events: &RwLock<Vec<StatsigEventInternal>>) {
        let count = match events.read().ok() {
            Some(e) => e.len(),
            _ => return,
        };

        let mut local_events = None;
        if count != 0 {
            if let Ok(mut lock) = events.write() {
                local_events = Some(std::mem::take(&mut *lock));
                drop(lock);
            }
        }

        if let Some(local_events) = local_events {
            let _ = network.send_events(local_events).await;
        }
    }

    fn spawn_bg_thread(&mut self) {
        let events = self.events.clone();
        let network = self.network.clone();
        let interval = Duration::from_millis(self.flush_interval_ms as u64);
        let is_shutdown = self.is_shutdown.clone();

        self.bg_thread_handle = Some(self.runtime_handle.spawn(async move {
            loop {
                if is_shutdown.load(Ordering::Relaxed) {
                    break;
                }
                Self::flush_impl(&network, &events).await;
                tokio::time::sleep(interval).await;
            }
        }));
    }
}
