use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use local_lru::LocalCache;
use store::s3_store::S3Store;
use store::s3_store::StoreError;
use tracing::info;
mod store;

struct Services {
    store: S3Store,
    cache: LocalCache,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_ansi(true).init();
    info!("Starting server");
    let store = S3Store::new("somebucket", "phi3").await;
    let cache = LocalCache::new(1000, 120);
    let services = Arc::new(Services { store, cache });
    let app = Router::new()
        .route("/keys/:key", get(get_key))
        .route("/keys/:key", post(post_key))
        .with_state(services);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Server listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn get_key(
    State(services): State<Arc<Services>>,
    Path(key): Path<String>,
) -> Result<Bytes, StatusCode> {
    if let Some(content) = services.cache.get_item(&key) {
        return Ok(content);
    }
    let res = services.store.get(&key).await;
    match res {
        Err(StoreError::ItemNotFound(key)) => {
            tracing::error!("Item {} not found", key);
            Err(StatusCode::NOT_FOUND)
        }
        Err(err) => {
            tracing::error!("Failed to get key: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(content) => {
            services.cache.add_item(&key, content.clone());
            Ok(content)
        }
    }
}

async fn post_key(
    State(services): State<Arc<Services>>,
    Path(key): Path<String>,
    payload: Bytes,
) -> StatusCode {
    let res = services.store.set(&key, payload).await;
    if res.is_err() {
        tracing::error!("Failed to set key: {}", res.err().unwrap());
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    StatusCode::CREATED
}
