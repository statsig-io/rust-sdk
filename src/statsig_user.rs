pub struct StatsigUser {
    pub user_id: Option<String>,
}

impl StatsigUser {
    pub fn new_with_user_id(user_id: String) -> StatsigUser {
        StatsigUser {
            user_id: Some(user_id)
        }
    }
}
