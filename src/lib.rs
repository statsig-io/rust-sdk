mod statsig;

// re-export public objects
pub use statsig::statsig_options::StatsigOptions;
pub use statsig::statsig_user::StatsigUser;

use statsig::statsig_driver::StatsigDriver;
use statsig::helpers::make_arc;

use std::borrow::Borrow;
use std::error::Error;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::statsig::statsig_error::StatsigError;

lazy_static! {
    static ref _instance: Arc<Mutex<Option<StatsigDriver>>> = make_arc(None);
}

pub struct Statsig {}

impl Statsig {
    pub async fn initialize(secret: &str, options: StatsigOptions) -> Option<StatsigError> {
        let mut mutex_guard = match _instance.lock().ok() {
            Some(guard) => guard,
            _ => {
                return Some(StatsigError::singleton_lock_failure());
            }
        };

        let mut driver = match mutex_guard.deref() {
            Some(d) => {
                return Some(StatsigError::singleton_lock_failure());
            }
            _ => StatsigDriver::new(secret, options)
        };
        driver.initialize().await;
        *mutex_guard = Some(driver);

        None
    }

    pub async fn check_gate(user: &StatsigUser, gate_name: &String) -> Result<bool, StatsigError> {
        match Self::check_gate_impl(user, gate_name).await {
            Some(result) => Ok(result),
            None => Err(StatsigError::uninitialized())
        }
    }

    async fn check_gate_impl(user: &StatsigUser, gate_name: &String) -> Option<bool> {
        Some(_instance.lock().ok()?.as_mut()?.check_gate(user, gate_name).await)
    }
}

