use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsigEvent {
    pub event_name: String,
    pub value: Option<Value>,
    pub metadata: Option<HashMap<String, Value>>,
}
