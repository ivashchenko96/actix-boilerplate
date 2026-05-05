use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::Settings;

/// Typesense client for search functionality
pub struct TypesenseClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl TypesenseClient {
    pub fn new(_settings: &Settings) -> Result<Self> {
        let api_key = std::env::var("TYPESENSE_API_KEY")
            .map_err(|_| anyhow::anyhow!("TYPESENSE_API_KEY not set"))?;

        let host = std::env::var("TYPESENSE_HOST").unwrap_or_else(|_| "localhost".to_string());

        let port = std::env::var("TYPESENSE_PORT").unwrap_or_else(|_| "8108".to_string());

        let base_url = format!("http://{}:{}", host, port);
        let client = Client::new();

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    pub async fn search(&self, collection: &str, query: &str) -> Result<SearchResponse> {
        let url = format!(
            "{}/collections/{}/documents/search",
            self.base_url, collection
        );

        let response = self
            .client
            .get(&url)
            .header("X-TYPESENSE-API-KEY", &self.api_key)
            .query(&[("q", query)])
            .send()
            .await?;

        let search_result: SearchResponse = response.json().await?;
        Ok(search_result)
    }

    pub async fn index_document<T: Serialize>(&self, collection: &str, document: &T) -> Result<()> {
        let url = format!("{}/collections/{}/documents", self.base_url, collection);

        self.client
            .post(&url)
            .header("X-TYPESENSE-API-KEY", &self.api_key)
            .json(document)
            .send()
            .await?;

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub found: u32,
}

#[derive(Deserialize)]
pub struct SearchHit {
    pub document: serde_json::Value,
}
