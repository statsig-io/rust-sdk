pub struct StatsigOptions {
    api_override: String,
}

impl StatsigOptions {
    pub fn new() -> StatsigOptions {
        StatsigOptions {
            api_override: String::from("")
        }
    }

    pub fn api_override(mut self, api_override: &str) -> StatsigOptions {
        self.api_override = String::from(api_override);
        self
    }
}

