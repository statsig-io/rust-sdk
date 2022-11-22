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
use crate::statsig::internal::{DynamicConfig, Layer};

mod statsig;

lazy_static! {
    static ref DRIVER: Arc<RwLock<Option<StatsigDriver>>> = Arc::from(RwLock::from(None));
}

pub struct Statsig {}

impl Statsig {
    pub async fn initialize(secret: &str, options: StatsigOptions) -> Option<StatsigError> {
        let mut guard = match DRIVER.write().ok() {
            Some(guard) => guard,
            _ => {
                return Some(StatsigError::singleton_lock_failure());
            }
        };

        let driver = match guard.deref() {
            Some(_d) => {
                return Some(StatsigError::already_initialized());
            }
            _ => match StatsigDriver::new(secret, options) {
                Ok(d) => d,
                Err(_e) => return Some(StatsigError::instantiation_failure())
            }
        };

        driver.initialize().await;
        *guard = Some(driver);

        None
    }

    pub async fn shutdown() -> Option<StatsigError> {
        let guard = match DRIVER.write().ok() {
            Some(guard) => guard,
            _ => {
                return Some(StatsigError::singleton_lock_failure());
            }
        };

        let driver = match guard.deref() {
            Some(d) => d,
            _ => return None
        };

        driver.shutdown().await;

        None
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

    fn use_driver<T>(func: impl FnOnce(&StatsigDriver) -> Result<T, StatsigError>) -> Result<T, StatsigError> {
        if let Some(guard) = DRIVER.read().ok() {
            if let Some(driver) = guard.deref() {
                return func(driver);
            }
            return Err(StatsigError::uninitialized());
        }
        Err(StatsigError::singleton_lock_failure())
    }
}

