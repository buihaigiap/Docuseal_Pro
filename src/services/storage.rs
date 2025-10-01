use aws_sdk_s3::{Client, primitives::ByteStream};
use chrono::Utc;

pub struct StorageService {
    client: Client,
    bucket: String,
}

impl StorageService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint = std::env::var("STORAGE_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:9000".to_string());
        let region = std::env::var("STORAGE_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());
        let bucket = std::env::var("STORAGE_BUCKET")
            .unwrap_or_else(|_| "docuseal".to_string());

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(aws_sdk_s3::config::Region::new(region))
            .credentials_provider(
                aws_sdk_s3::config::Credentials::new(
                    std::env::var("STORAGE_ACCESS_KEY_ID").unwrap_or_else(|_| "minioadmin".to_string()),
                    std::env::var("STORAGE_SECRET_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string()),
                    None,
                    None,
                    "minio-credentials",
                )
            )
            .load()
            .await;

        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&config);
        
        // Enable path style addressing for MinIO compatibility
        if std::env::var("STORAGE_USE_PATH_STYLE").unwrap_or_else(|_| "true".to_string()) == "true" {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }

        let s3_config = s3_config_builder.build();
        let client = Client::from_conf(s3_config);

        Ok(Self { client, bucket })
    }

    pub async fn upload_file(
        &self,
        file_data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = Utc::now().timestamp();
        let key = format!("templates/{}_{}", timestamp, filename);

        let byte_stream = ByteStream::from(file_data);

        match self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(byte_stream)
            .content_type(content_type)
            .send()
            .await {
            Ok(_) => Ok(key),
            Err(e) => {
                eprintln!("MinIO upload error: {:?}", e);
                Err(Box::new(e))
            }
        }
    }

    pub async fn download_file(
        &self,
        key: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = response.body.collect().await?;
        Ok(data.into_bytes().to_vec())
    }

    pub async fn delete_file(
        &self,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(())
    }

    pub async fn file_exists(
        &self,
        key: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}