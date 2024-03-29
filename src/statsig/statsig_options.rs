use std::collections::HashMap;

pub struct StatsigOptions {
    pub environment: Option<HashMap<String, String>>,
    pub api_override: String,
    pub api_for_download_config_specs: String,
    pub rulesets_sync_interval_ms: u32,
    pub logger_max_queue_size: u32,
    pub logger_flush_interval_ms: u32,
    pub disable_user_agent_support: bool,
}

impl StatsigOptions {
    pub fn default() -> StatsigOptions {
        StatsigOptions {
            environment: None,
            api_override: "https://statsigapi.net/v1".to_string(),
            api_for_download_config_specs: "https://api.statsigcdn.com/v1".to_string(),
            rulesets_sync_interval_ms: 10_000,
            logger_max_queue_size: 500,
            logger_flush_interval_ms: 60_000,
            disable_user_agent_support: false,
        }
    }
}
