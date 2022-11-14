mod statsig;

// re-export public objects to top level
pub use statsig::statsig_options::StatsigOptions;
pub use statsig::statsig_user::StatsigUser;

use std::ops::{Deref};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

use statsig::statsig_error::StatsigError;
use statsig::internal::driver::StatsigDriver;
use statsig::internal::helpers::make_arc;

lazy_static! {
    static ref INSTANCE: Arc<Mutex<Option<StatsigDriver>>> = make_arc(None);
}

pub struct Statsig {}

impl Statsig {
    pub async fn initialize(secret: &str, options: StatsigOptions) -> Option<StatsigError> {
        let mut mutex_guard = match INSTANCE.lock().ok() {
            Some(guard) => guard,
            _ => {
                return Some(StatsigError::singleton_lock_failure());
            }
        };

        let mut driver = match mutex_guard.deref() {
            Some(_d) => {
                return Some(StatsigError::already_initialized());
            }
            _ => StatsigDriver::new(secret, options)
        };
        driver.initialize().await;
        *mutex_guard = Some(driver);

        None
    }

    pub fn check_gate(user: &StatsigUser, gate_name: &String) -> Result<bool, StatsigError> {
        Self::use_instance(|driver| {
            Ok(driver.check_gate(user, gate_name))
        })
    }

    fn use_instance<T>(func: impl Fn(&StatsigDriver) -> Result<T, StatsigError>) -> Result<T, StatsigError> {
        if let Some(guard) = INSTANCE.lock().ok() {
            if let Some(driver) = guard.deref() {
                return func(driver);
            }
        }
        Err(StatsigError::uninitialized())
    }
}

