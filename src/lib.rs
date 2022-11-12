mod statsig;

// re-export public objects
pub use statsig::statsig_options::StatsigOptions;
pub use statsig::statsig_user::StatsigUser;

use statsig::statsig_driver::StatsigDriver;
use statsig::helpers::make_arc;

use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref DRIVER: Arc<Mutex<Option<StatsigDriver>>> = make_arc(None);
}

pub struct Statsig {}

impl Statsig {
    pub async fn initialize(secret: &str, options: StatsigOptions) {
        let mut driver = DRIVER.lock().unwrap();
        if driver.is_some() {
            println!("Statsig already initialized");
            return;
        }

        let mut new_driver = StatsigDriver::new(secret, options);
        new_driver.initialize().await;
        *driver = Some(new_driver);
    }

    pub async fn check_gate(user: &StatsigUser, gate_name: &String) -> bool {
        return DRIVER.lock().unwrap().as_mut().unwrap().check_gate(user, gate_name).await;
    }
}
