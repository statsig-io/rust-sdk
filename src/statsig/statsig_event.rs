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

