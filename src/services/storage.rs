use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use bytes::Bytes;

use crate::config::Settings;

/// Storage service for file uploads and management
pub struct StorageService {
    s3_client: Client,
    bucket_name: String,
}

impl StorageService {
    pub async fn new(_settings: &Settings) -> Result<Self> {
        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;
        let s3_client = Client::new(&config);

        let bucket_name =
            std::env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "default-bucket".to_string());

        Ok(Self {
            s3_client,
            bucket_name,
        })
    }

    pub async fn upload_file(&self, key: &str, data: Bytes, content_type: &str) -> Result<String> {
        self.s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await?;

        let url = format!("https://{}.s3.amazonaws.com/{}", self.bucket_name, key);
        Ok(url)
    }

    pub async fn delete_file(&self, key: &str) -> Result<()> {
        self.s3_client
            .delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_presigned_url(&self, key: &str, expires_in_secs: u64) -> Result<String> {
        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::expires_in(
            std::time::Duration::from_secs(expires_in_secs),
        )?;

        let presigned_request = self
            .s3_client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key)
            .presigned(presigning_config)
            .await?;

        Ok(presigned_request.uri().to_string())
    }
}
