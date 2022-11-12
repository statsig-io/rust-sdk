pub struct StatsigOptions {
    pub api_override: String,
}

impl StatsigOptions {
    pub fn default() -> StatsigOptions {
        StatsigOptions {
            api_override: "https://statsigapi.net/v1".to_string()
        }
    }
}

