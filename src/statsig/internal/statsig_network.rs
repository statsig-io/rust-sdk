use std::collections::HashMap;

use http::{HeaderMap};
use reqwest::{Client, Response, Error};
use serde_json::{json, Value};
use crate::{StatsigEvent, StatsigOptions};
use crate::statsig::statsig_event::StatsigEventInternal;

use super::data_types::APIDownloadedConfigs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct StatsigNetwork {
    client: Client,
    secret: String,
    base_api: String,
    statsig_metadata: Value,
}

impl StatsigNetwork {
    pub fn new(secret_key: &str, options: &StatsigOptions) -> Self {
        StatsigNetwork {
            client: Client::new(),
            secret: secret_key.to_string(),
            base_api: options.api_override.clone(),
            statsig_metadata: json!(HashMap::from([
                ("sdkType".to_string(), "".to_string()),
                ("sdkVersion".to_string(), VERSION.to_string())
            ])),
        }
    }

    pub async fn download_config_specs(&self) -> Option<APIDownloadedConfigs> {
        let mut body = HashMap::new();
        body.insert("lang", json!("rust"));
        body.insert("body", json!("json"));

        let res = match self.make_request("download_config_specs", &mut body)
            .await.ok() {
            Some(x) => x,
            None => return None
        };

        if res.status() != 200 {
            return None;
        }

        res.json::<APIDownloadedConfigs>().await.ok()
    }

    pub async fn send_events(&self, events: &Vec<StatsigEventInternal>) {
        let mut body: HashMap<&str, Value> = HashMap::new();
        body.insert("events", json!(events));

        let res = match self.make_request("log_event", &mut body)
            .await.ok() {
            Some(x) => x,
            None => return
        };

        println!("{}", res.status())
    }

    async fn make_request(&self, endpoint: &str, body: &mut HashMap<&str, Value>) -> Result<Response, Error> {
        let url = format!("{}/{}", self.base_api, endpoint);

        let mut headers = HeaderMap::new();
        headers.insert("STATSIG-API-KEY", self.secret.parse().unwrap());

        body.insert("statsigMetadata", self.statsig_metadata.clone());

        self.client.post(url)
            .json(&body)
            .headers(headers)
            .send().await
    }
}