use std::collections::HashMap;

use http::HeaderMap;
use reqwest::{Client, Error, Response};
use serde_json::{json, Value};

use crate::StatsigOptions;
use crate::statsig::internal::statsig_event_internal::StatsigEventInternal;

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
                ("sdkType".to_string(), "rust-server".to_string()),
                ("sdkVersion".to_string(), VERSION.to_string())
            ])),
        }
    }

    pub async fn download_config_specs(&self, since_time: u64) -> Option<APIDownloadedConfigs> {
        let mut body = HashMap::new();
        body.insert("sinceTime", json!(since_time));

        let res = self.make_request("download_config_specs", &mut body).await.ok()?;

        if res.status() != 200 {
            return None;
        }

        match res.json().await {
            Ok(json) => Some(json),
            Err(e) => {
                println!("[Statsig] Parsing Error: {}", e);
                None
            }
        }
    }

    pub async fn send_events(&self, events: Vec<StatsigEventInternal>) -> Option<Response> {
        let mut body = HashMap::from([
            ("events", json!(events))
        ]);

        return self.make_request("log_event", &mut body)
            .await.ok();
    }

    async fn make_request(&self, endpoint: &str, body: &mut HashMap<&str, Value>) -> Result<Response, Error> {
        let url = format!("{}/{}", self.base_api, endpoint);

        let mut headers = HeaderMap::new();
        headers.insert("STATSIG-API-KEY", self.secret.parse().expect("statsig_api_key -> header"));

        body.insert("statsigMetadata", self.statsig_metadata.clone());

        self.client.post(url)
            .json(&body)
            .headers(headers)
            .send().await
    }
}