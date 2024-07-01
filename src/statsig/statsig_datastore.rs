pub const CONFIG_SPEC_KEY: &str = "statsig.cache";

pub trait StatsigDatastore: Send + Sync {
    fn initialize(&self);
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn shutdown(&self);

    // Returns whether this datastore should be used instead of the Statsig network for
    // periodically updating config specs.
    fn should_be_used_for_querying_updates(&self) -> bool {
        true
    }
}
