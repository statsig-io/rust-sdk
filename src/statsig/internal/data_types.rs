use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct APISpec {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub salt: String,
    pub default_value: Value,
    pub enabled: bool,
    pub rules: Vec<APIRule>,
    pub id_type: String,
    pub explicit_parameters: Option<Vec<String>>,
    pub entity: String,
    pub has_shared_params: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct APIRule {
    pub name: String,
    pub pass_percentage: f64,
    pub return_value: Value,
    pub id: String,
    pub salt: Option<String>,
    pub conditions: Vec<APICondition>,
    pub id_type: String,
    pub group_name: Option<String>,
    pub config_delegate: Option<String>,
    pub is_experiment_group: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct APICondition {
    #[serde(rename = "type")]
    pub condition_type: String,
    pub target_value: Option<Value>,
    pub operator: Option<String>,
    pub field: Option<String>,
    pub additional_values: Option<HashMap<String, Value>>,
    pub id_type: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct APIDownloadedConfigsWithUpdates {
    pub feature_gates: Vec<APISpec>,
    pub dynamic_configs: Vec<APISpec>,
    pub layer_configs: Vec<APISpec>,
    pub id_lists: Option<HashMap<String, bool>>,
    pub layers: Option<HashMap<String, Vec<String>>>,
    pub has_updates: bool,
    pub time: u64,
}

#[derive(Deserialize)]
pub struct APIDownloadedConfigsNoUpdates {
    pub has_updates: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum APIDownloadedConfigsResponse {
    WithUpdates(APIDownloadedConfigsWithUpdates),
    NoUpdates(APIDownloadedConfigsNoUpdates),
}
