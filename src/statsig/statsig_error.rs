pub struct StatsigError {
    pub message: String,
}


impl StatsigError {
    pub fn singleton_lock_failure() -> Self {
        StatsigError {
            message: "Failed to acquire mutex lock on Statsig instance".to_string()
        }
    }

    pub fn already_initialized() -> Self {
        StatsigError {
            message: "Statsig is already initialized".to_string()
        }
    }

    pub fn uninitialized() -> Self {
        StatsigError {
            message: "You must call and await Statsig.initialize first.".to_string()
        }
    }
}
