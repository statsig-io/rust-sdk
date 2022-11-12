use std::collections::HashMap;

pub struct StatsigUser {
    pub user_id: Option<String>,
    pub custom_ids: Option<HashMap<String, String>>,
    pub email: Option<String>
}

impl StatsigUser {
    pub fn with_user_id(user_id: String) -> Self {
        StatsigUser {
            user_id: Some(user_id),
            ..Self::default()
        }
    }
    
    fn default() -> Self {
        StatsigUser {
            user_id: None,
            custom_ids: None,
            email: None
        }
    }
}
