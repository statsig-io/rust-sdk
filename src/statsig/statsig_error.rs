use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatsigError {
    #[error("Failed to acquire mutex lock on Statsig instance")]
    SingletonLockFailure,
    #[error("Statsig is already initialized")]
    AlreadyInitialized,
    #[error("Failed to create a Statsig instance")]
    InstantiationFailure,
    #[error("You must call and await Statsig.initialize first.")]
    Uninitialized,
    #[error("Was unable to gracefully shutdown the Statsig instance")]
    ShutdownFailure,
}
