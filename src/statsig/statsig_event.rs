use serde_json::Value;

use crate::StatsigUser;

pub struct StatsigEvent {
    pub user: StatsigUser,
    pub event_name: String,
    pub value: Value,
}

impl StatsigEvent {}