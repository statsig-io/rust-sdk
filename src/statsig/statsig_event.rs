use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::StatsigUser;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsigEvent {
    pub user: StatsigUser,
    pub event_name: String,
    pub value: Option<Value>,
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsigEventInternal {
    #[serde(flatten)]
    pub event_data: StatsigEvent,
    pub time: u64,
    pub secondary_exposures: Vec<HashMap<String, String>>,
}

impl StatsigEventInternal {
    pub fn from_event(event: StatsigEvent) -> Self {
        Self {
            event_data: event,
            time: 1,
            secondary_exposures: vec![]
        }
    }
}



