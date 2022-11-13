use std::any::Any;
use std::collections::HashMap;
use std::fmt::Error;
use reqwest::Client;
use http::{HeaderMap};

use crate::StatsigOptions;

use super::data_types::APIDownloadedConfigs;

pub struct StatsigNetwork {
    client: Client,
    secret: String,
    base_api: String,
}

impl StatsigNetwork {
    pub fn new(secret_key: &str, options: &StatsigOptions) -> Self {
        StatsigNetwork { client: Client::new(), secret: secret_key.to_string(), base_api: options.api_override.clone() }
    }

    pub async fn download_config_specs(&self) -> Result<APIDownloadedConfigs, &str> {
        let mut headers = HeaderMap::new();
        headers.insert("STATSIG-API-KEY", self.secret.parse().unwrap());

        let mut body = HashMap::new();
        body.insert("lang", "rust");
        body.insert("body", "json");

        let url = format!("{}/download_config_specs", self.base_api);
        let res = match self.client.post(url)
            .json(&body)
            .headers(headers)
            .send().await.ok() {
            Some(x) => x,
            None => return Err("Request Failed")
        };

        if res.status() != 200 {
            return Err("Request Failed");
        }

        match res.json::<APIDownloadedConfigs>()
            .await.ok() {
            Some(x) => Ok(x),
            None => Err("Failed to Parse Response")
        }
    }
}