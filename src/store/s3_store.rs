use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    error::SdkError,
    operation::{get_object::GetObjectError, put_object::PutObjectError},
    primitives::ByteStreamError,
    Client,
};
use bytes::Bytes;
use thiserror::Error;

pub struct S3Store {
    client: Client,
    bucket: String,
    store: String,
}

impl S3Store {
    pub async fn new(bucket: &str, store: &str) -> Self {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = aws_sdk_s3::Client::new(&config);
        Self {
            client,
            bucket: bucket.to_string(),
            store: store.to_string(),
        }
    }

    pub async fn set(&self, key: &str, body: Bytes) -> Result<(), StoreError> {
        let payload = aws_sdk_s3::primitives::ByteStream::from(body);
        let key = format!("{}/{}", self.store, key);
        self.client
            .put_object()
            .bucket(self.bucket.clone())
            .key(key)
            .body(payload)
            .send()
            .await
            .map_err(StoreError::from)?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Bytes, StoreError> {
        let key = format!("{}/{}", self.store, key);

        let res = self
            .client
            .get_object()
            .bucket(self.bucket.clone())
            .key(key.clone())
            .send()
            .await;

        if let Err(e) = &res {
            match e {
                SdkError::ServiceError(service_error) => {
                    if matches!(service_error.err(), GetObjectError::NoSuchKey(_)) {
                        tracing::warn!("S3 key: {} not found", &key);
                        return Err(StoreError::ItemNotFound(key));
                    }
                }
                _ => return Err(StoreError::S3ReadError(e.to_string())),
            }
        }

        let body = res?.body.collect().await?;
        Ok(body.into_bytes())
    }
}

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("S3 write operation failed: {0}")]
    S3WriteError(String),
    #[error("S3 read operation failed: {0}")]
    S3ReadError(String),
    #[error("Item {0} not found")]
    ItemNotFound(String),
}
impl From<SdkError<PutObjectError>> for StoreError {
    fn from(err: SdkError<PutObjectError>) -> Self {
        StoreError::S3WriteError(err.to_string())
    }
}
impl From<SdkError<GetObjectError>> for StoreError {
    fn from(err: SdkError<GetObjectError>) -> Self {
        StoreError::S3ReadError(err.to_string())
    }
}
impl From<ByteStreamError> for StoreError {
    fn from(err: ByteStreamError) -> Self {
        StoreError::S3ReadError(err.to_string())
    }
}
