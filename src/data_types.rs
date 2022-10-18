use std::any::Any;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::{Number, Value};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub salt: String,
    pub default_value: Value,
    pub enabled: bool,
    pub rules: Vec<APIRule>,
    pub id_type: String,
    pub explicit_parameters: Option<Vec<String>>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APIRule {
    pub name: String,
    pub pass_percentage: Number,
    pub return_value: Value,
    pub id: String,
    pub salt: Option<String>,
    pub conditions: Vec<APICondition>,
    pub id_type: String,
    pub group_name: String,
    pub config_delegate: Option<String>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct APICondition {
    #[serde(rename = "type")]
    pub _type: String,
    pub target_value: Option<Value>,
    pub operator: Option<String>,
    pub field: Option<String>,
    pub additional_values: Option<HashMap<String, Value>>,
    pub id_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct APIDownloadedConfigs {
    pub feature_gates: Vec<APIConfig>,
    pub dynamic_configs: Vec<APIConfig>,
    pub layer_configs: Vec<APIConfig>,
    pub id_lists: Option<HashMap<String, bool>>,
    pub layers: Option<HashMap<String, Vec<String>>>,
    pub has_updates: bool,
    pub time: Number
}
