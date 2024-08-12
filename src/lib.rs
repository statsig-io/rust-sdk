extern crate core;

use std::ops::Deref;
use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;
use serde::de::DeserializeOwned;
use serde_json::Value;

use statsig::internal::StatsigDriver;
use statsig::statsig_error::StatsigError;
//
// re-export public objects to top level
pub use statsig::statsig_datastore::StatsigDatastore;
pub use statsig::statsig_event::StatsigEvent;
pub use statsig::statsig_options::StatsigOptions;
pub use statsig::statsig_user::StatsigUser;
use tokio::task::spawn_blocking;

use crate::statsig::internal::{DynamicConfig, Layer, LayerLogData, feature_gate::FeatureGate};

mod statsig;

lazy_static! {
    static ref DRIVER: Arc<RwLock<Option<StatsigDriver>>> = Arc::from(RwLock::from(None));
}

pub struct Statsig {}

impl Statsig {
    pub async fn initialize(secret: &str) -> Option<StatsigError> {
        Self::initialize_with_options(secret, StatsigOptions::default()).await
    }

    pub async fn initialize_with_options(
        secret: &str,
        options: StatsigOptions,
    ) -> Option<StatsigError> {
        match DRIVER.read().ok() {
            Some(read_guard) => {
                if read_guard.is_some() {
                    return Some(StatsigError::AlreadyInitialized);
                }
            }
            None => {
                return Some(StatsigError::SingletonLockFailure);
            }
        }

        let driver = unwrap_or_return!(
            StatsigDriver::new(secret, options).ok(),
            Some(StatsigError::InstantiationFailure)
        );

        driver.initialize().await;

        let mut write_guard = unwrap_or_return!(
            DRIVER.write().ok(),
            Some(StatsigError::SingletonLockFailure)
        );

        *write_guard = Some(driver);
        None
    }

    pub async fn shutdown() -> Option<StatsigError> {
        let driver_clone = Arc::clone(&DRIVER);
        match spawn_blocking(move || {
            let mut write_guard = unwrap_or_return!(
                driver_clone.write().ok(),
                Err(StatsigError::SingletonLockFailure)
            );

            if let Some(driver) = write_guard.take() {
                driver.shutdown();
            }
            Ok(())
        })
        .await
        {
            Ok(_t) => None,
            Err(_e) => Some(StatsigError::ShutdownFailure),
        }
    }

    pub fn check_gate(user: &StatsigUser, gate_name: &str) -> Result<bool, StatsigError> {
        Self::use_driver(|driver| Ok(driver.check_gate(user, gate_name)))
    }

    pub fn get_feature_gate(user: &StatsigUser, gate_name: &str) -> Result<FeatureGate, StatsigError> {
        Self::use_driver(|driver| Ok(driver.get_feature_gate(user, gate_name)))
    }

    pub fn get_config<T: DeserializeOwned>(
        user: &StatsigUser,
        config_name: &str,
    ) -> Result<DynamicConfig<T>, StatsigError> {
        Self::use_driver(|driver| Ok(driver.get_config(user, config_name)))
    }

    pub fn get_experiment<T: DeserializeOwned>(
        user: &StatsigUser,
        experiment_name: &str,
    ) -> Result<DynamicConfig<T>, StatsigError> {
        Self::get_config(user, experiment_name)
    }

    pub fn get_layer(user: &StatsigUser, layer_name: &str) -> Result<Layer, StatsigError> {
        Self::use_driver(|driver| Ok(driver.get_layer(user, layer_name)))
    }

    pub fn log_event(user: &StatsigUser, event: StatsigEvent) -> Option<StatsigError> {
        let res = Self::use_driver(move |driver| {
            driver.log_event(user, event);
            Ok(())
        });

        match res {
            Err(e) => Some(e),
            _ => None,
        }
    }

    pub fn get_client_initialize_response(user: &StatsigUser) -> Result<Value, StatsigError> {
        Self::use_driver(|driver| Ok(driver.get_client_initialize_response(user)))
    }

    pub(crate) fn log_layer_parameter_exposure(
        layer: &Layer,
        parameter_name: &str,
        log_data: &LayerLogData,
    ) {
        let _ = Self::use_driver(|driver| {
            driver.log_layer_parameter_exposure(layer, parameter_name, log_data);
            Ok(())
        });
    }

    fn use_driver<T>(
        func: impl FnOnce(&StatsigDriver) -> Result<T, StatsigError>,
    ) -> Result<T, StatsigError> {
        if let Ok(guard) = DRIVER.read() {
            if let Some(driver) = guard.deref() {
                return func(driver);
            }
            return Err(StatsigError::Uninitialized);
        }
        Err(StatsigError::SingletonLockFailure)
    }

    #[doc(hidden)]
    #[cfg(statsig_kong)]
    pub async fn __unsafe_reset() {
        if let Some(mut guard) = DRIVER.write().ok() {
            if let Some(driver) = guard.take() {
                let _ = spawn_blocking(move || {
                    driver.shutdown();
                })
                .await;
            }
        }
    }
}
