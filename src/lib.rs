extern crate core;

pub mod statsig_options;
pub mod statsig_driver;
pub mod statsig_user;

mod statsig_network;
mod statsig_store;
mod data_types;
mod statsig_evaluator;
mod helpers;

use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use crate::helpers::make_arc;
use crate::statsig_driver::*;
use crate::statsig_options::*;
use crate::statsig_user::StatsigUser;

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
        // 
        // let guard = DRIVER_MUTEX.lock().unwrap();
        // if guard.is_some() {
        //     guard.unwrap().initialize()
        // }
        // match guard {
        //     Some(x) => x.initialize(),
        //     None => false
        // };
    }

    pub async fn check_gate(user: &StatsigUser, gate_name: &String) -> bool {
        return DRIVER.lock().unwrap().as_mut().unwrap().check_gate(user, gate_name).await;
        // match driver {
        //     Some(x) => print!("foo"),
        //     _ => println!("")
        // }
        // .unwrap().check_gate(user, gate_name).await;
        return false;
        // let mut driver = DRIVER.lock().unwrap();
        // if driver.is_none() {
        //     println!("Statsig not initialized");
        //     return false
        // }
        // return &driver.unwrap().check_gate(user, gate_name).await;
        // return match DRIVER.lock().unwrap() {
        //     Some(x) => x.check_gate(user, gate_name),
        //     None => false
        // };
    }
}
