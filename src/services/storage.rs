use aws_sdk_s3::{Client, primitives::ByteStream};
use chrono::Utc;
use std::fs;
use std::path::Path;

pub struct StorageService {
    storage_type: String,
    local_path: Option<String>,
    client: Option<Client>,
    bucket: Option<String>,
}

impl StorageService {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let storage_type = std::env::var("STORAGE_TYPE").unwrap_or_else(|_| "s3".to_string());
        println!("=== STORAGE DEBUG ===");
        println!("STORAGE_TYPE env var: {:?}", std::env::var("STORAGE_TYPE"));
        println!("Using storage_type: {}", storage_type);
        println!("====================");

        if storage_type == "local" {
            let local_path = std::env::var("STORAGE_PATH").unwrap_or_else(|_| "./uploads".to_string());
            fs::create_dir_all(&local_path)?;
            Ok(Self {
                storage_type,
                local_path: Some(local_path),
                client: None,
                bucket: None,
            })
        } else {
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

            Ok(Self {
                storage_type,
                local_path: None,
                client: Some(client),
                bucket: Some(bucket),
            })
        }
    }

    pub async fn upload_file(
        &self,
        file_data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = Utc::now().timestamp();
        let key = format!("templates/{}_{}", timestamp, filename);

        if self.storage_type == "local" {
            let path = Path::new(self.local_path.as_ref().unwrap()).join(&key);
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, &file_data)?;
            Ok(key)
        } else {
            let byte_stream = ByteStream::from(file_data);

            match self.client.as_ref().unwrap()
                .put_object()
                .bucket(self.bucket.as_ref().unwrap())
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
    }

    pub async fn download_file(
        &self,
        key: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if self.storage_type == "local" {
            let path = Path::new(self.local_path.as_ref().unwrap()).join(key);
            let data = fs::read(&path)?;
            Ok(data)
        } else {
            let response = self.client.as_ref().unwrap()
                .get_object()
                .bucket(self.bucket.as_ref().unwrap())
                .key(key)
                .send()
                .await?;

            let data = response.body.collect().await?;
            Ok(data.into_bytes().to_vec())
        }
    }

    pub async fn delete_file(
        &self,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.storage_type == "local" {
            let path = Path::new(self.local_path.as_ref().unwrap()).join(key);
            fs::remove_file(&path)?;
            Ok(())
        } else {
            self.client.as_ref().unwrap()
                .delete_object()
                .bucket(self.bucket.as_ref().unwrap())
                .key(key)
                .send()
                .await?;

            Ok(())
        }
    }

    pub async fn file_exists(
        &self,
        key: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if self.storage_type == "local" {
            let path = Path::new(self.local_path.as_ref().unwrap()).join(key);
            Ok(path.exists())
        } else {
            match self.client.as_ref().unwrap()
                .head_object()
                .bucket(self.bucket.as_ref().unwrap())
                .key(key)
                .send()
                .await
            {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
    }

    pub fn get_public_url(&self, key: &str) -> String {
        if self.storage_type == "local" {
            format!("/api/files/{}", key)
        } else {
            let endpoint = std::env::var("STORAGE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string());
            let bucket = self.bucket.as_ref().unwrap();
            format!("{}/{}/{}", endpoint, bucket, key)
        }
    }
}