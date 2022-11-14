pub struct StatsigOptions {
    pub api_override: String,
    pub rulesets_sync_interval_ms: u32,
    pub logger_max_queue_size: u32,
    pub logger_flush_interval_ms: u32,
}

impl StatsigOptions {
    pub fn default() -> StatsigOptions {
        StatsigOptions {
            api_override: "https://statsigapi.net/v1".to_string(),
            rulesets_sync_interval_ms: 10_000,
            logger_max_queue_size: 500,
            logger_flush_interval_ms: 60_000,
        }
    }
}

