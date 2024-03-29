use std::collections::HashMap;

use crate::statsig::internal::data_types::APIDownloadedConfigsResponse::{NoUpdates, WithUpdates};
use crate::statsig::internal::data_types::{
    APIDownloadedConfigsNoUpdates, APIDownloadedConfigsResponse, APIDownloadedConfigsWithUpdates,
};
use http::HeaderMap;
use reqwest::{Client, Error, Response};
use serde_json::{from_value, json, Value};

use crate::statsig::internal::statsig_event_internal::StatsigEventInternal;
use crate::StatsigOptions;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct StatsigNetwork {
    client: Client,
    secret: String,
    base_api: String,
    dcs_api: String,
    statsig_metadata: Value,
}

impl StatsigNetwork {
    pub fn new(secret_key: &str, options: &StatsigOptions) -> Self {
        StatsigNetwork {
            client: Client::new(),
            secret: secret_key.to_string(),
            base_api: options.api_override.clone(),
            dcs_api: options.api_for_download_config_specs.clone(),
            statsig_metadata: json!(HashMap::from([
                ("sdkType".to_string(), "rust-server".to_string()),
                ("sdkVersion".to_string(), VERSION.to_string())
            ])),
        }
    }

    pub async fn download_config_specs(
        &self,
        since_time: u64,
    ) -> Option<APIDownloadedConfigsResponse> {
        let res = match self.dcs_api == "https://api.statsigcdn.com/v1" {
            true => self
                .make_get_request(&format!(
                    "download_config_specs/{}.json?sinceTime={}",
                    self.secret, since_time
                ))
                .await
                .ok()?,
            false => {
                let mut body = HashMap::new();
                body.insert("sinceTime", json!(since_time));
                self.make_post_request("download_config_specs", &mut body)
                    .await
                    .ok()?
            }
        };

        if res.status().as_u16() > 299 {
            println!(
                "[Statsig] Unexpected status code ({}) for download_config_specs.",
                res.status()
            );
            return None;
        }

        let text = res.text().await.ok()?;
        let json_value: Value = serde_json::from_str(&text).ok()?;
        if let Ok(with_updates) = from_value::<APIDownloadedConfigsWithUpdates>(json_value.clone())
        {
            return Some(WithUpdates(with_updates));
        }

        if let Ok(no_updates) = from_value::<APIDownloadedConfigsNoUpdates>(json_value.clone()) {
            if no_updates.has_updates == false {
                return Some(NoUpdates(no_updates));
            }
        }

        None
    }

    pub async fn send_events(&self, events: Vec<StatsigEventInternal>) -> Option<Response> {
        let mut body = HashMap::from([("events", json!(events))]);

        self.make_post_request("log_event", &mut body).await.ok()
    }

    fn get_api_url(&self, endpoint: &str) -> String {
        let api = match endpoint.starts_with("download_config_specs") {
            true => self.dcs_api.clone(),
            false => self.base_api.clone(),
        };
        match api.ends_with('/') {
            true => format!("{}{}", api, endpoint),
            false => format!("{}/{}", api, endpoint),
        }
    }

    async fn make_get_request(&self, endpoint: &str) -> Result<Response, Error> {
        self.client.get(self.get_api_url(endpoint)).send().await
    }

    async fn make_post_request(
        &self,
        endpoint: &str,
        body: &mut HashMap<&str, Value>,
    ) -> Result<Response, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "STATSIG-API-KEY",
            self.secret.parse().expect("statsig_api_key -> header"),
        );

        body.insert("statsigMetadata", self.statsig_metadata.clone());

        self.client
            .post(self.get_api_url(endpoint))
            .json(&body)
            .headers(headers)
            .send()
            .await
    }
}
