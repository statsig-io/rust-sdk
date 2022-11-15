use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsigUser {
    #[serde(rename = "userID")]
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub country: Option<String>,
    pub locale: Option<String>,
    pub app_version: Option<String>,
    pub custom: Option<HashMap<String, Value>>,
    pub private_attributes: Option<HashMap<String, Value>>,
    #[serde(rename = "customIDs")]
    pub custom_ids: Option<HashMap<String, String>>,
    
    pub(crate) statsig_environment: Option<HashMap<String, String>>,
}

impl StatsigUser {
    pub fn with_user_id(user_id: String) -> Self {
        StatsigUser {
            user_id: Some(user_id),
            ..Self::default()
        }
    }
    
    pub fn with_custom_ids(custom_ids: HashMap<String, String>) -> Self {
        StatsigUser {
            custom_ids: Some(custom_ids),
            ..Self::default()
        }
    }
    
    fn default() -> Self {
        StatsigUser {
            user_id: None,
            email: None,
            ip: None,
            user_agent: None,
            country: None,
            locale: None,
            app_version: None,
            custom: None,
            private_attributes: None,
            custom_ids: None,
            statsig_environment: None
        }
    }
}
