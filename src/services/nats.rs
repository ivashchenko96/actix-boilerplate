use async_nats::{Client, jetstream::Context};
use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::config::Settings;

/// NATS service for event streaming and messaging
pub struct NatsService {
    client: Client,
    jetstream: Context,
}

impl NatsService {
    pub async fn new(settings: &Settings) -> Result<Self> {
        let nats_url = std::env::var("NATS_URL")
            .unwrap_or_else(|_| "nats://localhost:4222".to_string());

        let client = async_nats::connect(&nats_url).await?;
        let jetstream = async_nats::jetstream::new(client.clone());

        Ok(Self { client, jetstream })
    }

    pub async fn publish(&self, subject: &str, payload: impl Serialize) -> Result<()> {
        let data = serde_json::to_vec(&payload)?;
        self.client.publish(subject, data.into()).await?;
        Ok(())
    }

    pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber> {
        let subscriber = self.client.subscribe(subject).await?;
        Ok(subscriber)
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn jetstream(&self) -> &Context {
        &self.jetstream
    }
}