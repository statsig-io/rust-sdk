extern crate core;

use std::ops::Deref;
use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;

use statsig::internal::StatsigDriver;
use statsig::statsig_error::StatsigError;
//
// re-export public objects to top level
pub use statsig::statsig_event::StatsigEvent;
pub use statsig::statsig_options::StatsigOptions;
pub use statsig::statsig_user::StatsigUser;
use tokio::task::spawn_blocking;

use crate::statsig::internal::{DynamicConfig, Layer, LayerLogData};

mod statsig;

lazy_static! {
    static ref DRIVER: Arc<RwLock<Option<StatsigDriver>>> = Arc::from(RwLock::from(None));
}

pub struct Statsig {}

impl Statsig {
    pub async fn initialize(secret: &str, options: StatsigOptions) -> Option<StatsigError> {
        let read_guard = unwrap_or_return!(
            DRIVER.read().ok(), Some(StatsigError::singleton_lock_failure()));

        if let Some(_driver) = read_guard.deref() {
            return Some(StatsigError::already_initialized());
        }
        drop(read_guard);

        let driver = unwrap_or_return!(
            StatsigDriver::new(secret, options).ok(), Some(StatsigError::instantiation_failure()));

        driver.initialize().await;

        let mut write_guard = unwrap_or_return!(
            DRIVER.write().ok(), Some(StatsigError::singleton_lock_failure()));

        *write_guard = Some(driver);
        None
    }

    pub async fn shutdown() -> Option<StatsigError> {
        let mut write_guard = unwrap_or_return!(
            DRIVER.write().ok(), Some(StatsigError::singleton_lock_failure()));

        let driver = unwrap_or_return!(write_guard.take(), None);
        match spawn_blocking(move || {
            driver.shutdown();
        }).await {
            Ok(_t) => None,
            Err(_e) => Some(StatsigError::shutdown_failure())
        }
    }

    pub fn check_gate(user: StatsigUser, gate_name: &String) -> Result<bool, StatsigError> {
        Self::use_driver(|driver| {
            Ok(driver.check_gate(user, gate_name))
        })
    }

    pub fn get_config(user: StatsigUser, config_name: &String) -> Result<DynamicConfig, StatsigError> {
        Self::use_driver(|driver| {
            Ok(driver.get_config(user, config_name))
        })
    }

    pub fn get_experiment(user: StatsigUser, experiment_name: &String) -> Result<DynamicConfig, StatsigError> {
        Self::get_config(user, experiment_name)
    }


    pub fn get_layer(user: StatsigUser, layer_name: &String) -> Result<Layer, StatsigError> {
        Self::use_driver(|driver| {
            Ok(driver.get_layer(user, layer_name))
        })
    }

    pub fn log_event(event: StatsigEvent) -> Option<StatsigError> {
        let res = Self::use_driver(move |driver| {
            Ok(driver.log_event(event))
        });

        match res {
            Err(e) => Some(e),
            _ => None
        }
    }

    pub(crate) fn log_layer_parameter_exposure(layer: &Layer, parameter_name: &String, log_data: &LayerLogData) {
        let _ = Self::use_driver(|driver| {
            Ok(driver.log_layer_parameter_exposure(layer, parameter_name, log_data))
        });
    }

    fn use_driver<T>(func: impl FnOnce(&StatsigDriver) -> Result<T, StatsigError>) -> Result<T, StatsigError> {
        if let Some(guard) = DRIVER.read().ok() {
            if let Some(driver) = guard.deref() {
                return func(driver);
            }
            return Err(StatsigError::uninitialized());
        }
        Err(StatsigError::singleton_lock_failure())
    }

    #[doc(hidden)]
    #[cfg(statsig_kong)]
    pub async fn __unsafe_reset() {
        if let Some(mut guard) = DRIVER.write().ok() {
            if let Some(driver) = guard.take() {
                let _ = spawn_blocking(move || {
                    driver.shutdown();
                }).await;
            }
        }
    }
}

