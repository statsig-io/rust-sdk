use std::collections::HashMap;

pub struct StatsigUser {
    pub user_id: Option<String>,
    pub custom_ids: Option<HashMap<String, String>>,
}

impl StatsigUser {
    pub fn new_with_user_id(user_id: String) -> StatsigUser {
        StatsigUser {
            user_id: Some(user_id),
            custom_ids: None
        }
    }
}
