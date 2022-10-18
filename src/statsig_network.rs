use crate::data_types::APIDownloadedConfigs;

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Error;
use reqwest::Client;
use http::{HeaderMap};

pub struct StatsigNetwork {
    client: Client,
}

impl StatsigNetwork {
    pub fn new() -> StatsigNetwork {
        StatsigNetwork { client: Client::new() }
    }

    pub async fn download_config_specs(&self) -> Result<APIDownloadedConfigs, &str> {
        let mut headers = HeaderMap::new();
        headers.insert("STATSIG-API-KEY", "secret-xxx".parse().unwrap());

        let mut body = HashMap::new();
        body.insert("lang", "rust");
        body.insert("body", "json");

        let res = match self.client.post("https://statsigapi.net/v1/download_config_specs")
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