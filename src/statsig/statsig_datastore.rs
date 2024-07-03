use async_trait::async_trait;

pub const CONFIG_SPEC_KEY: &str = "statsig.cache";

#[async_trait] // when implementing this trait, use the #[async_trait] macro
pub trait StatsigDatastore: Send + Sync {
    async fn initialize(&self);
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: &str);
    async fn shutdown(&self);

    // Returns whether this datastore should be used instead of the Statsig network for
    // periodically updating config specs.
    fn should_be_used_for_querying_updates(&self) -> bool {
        true
    }
}
