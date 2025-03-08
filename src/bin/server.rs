use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use blockchain::Blockchain;
use domi_blockchain::blockchain; // Correct crate name
use serde_json::Value;
use std::sync::{Arc, RwLock};
#[tokio::main]
async fn main() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));

    let app = Router::new()
        .route("/secret/:did", get(get_secret))
        .with_state(blockchain.clone())
        .route("/pubkey/:did", get(get_pubkey))
        .with_state(blockchain.clone())
        .route("/did/new", post(add_block))
        .with_state(blockchain.clone())
        .route("/blockchain", get(get_blockchain))
        .with_state(blockchain.clone());

    let addr = "0.0.0.0:3000".parse().unwrap();
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    async fn get_secret(
        State(blockchain): State<Arc<RwLock<Blockchain>>>,
        Path(did): Path<String>,
    ) -> impl IntoResponse {
        blockchain
            .read()
            .unwrap()
            .generate_challenge(did)
            .to_string()
    }

    async fn get_pubkey(
        State(blockchain): State<Arc<RwLock<Blockchain>>>,
        Path(did): Path<String>,
    ) -> String {
        blockchain
            .read()
            .unwrap()
            .get_latest_public_key(did)
            .to_string()
    }

    async fn add_block(
        State(blockchain): State<Arc<RwLock<Blockchain>>>,
        Json(payload): Json<Value>,
    ) {
        let did = payload["did"].as_str().unwrap_or("").to_string();
        let pubkey = payload["pubkey"].as_str().unwrap_or("").to_string();

        blockchain.write().unwrap().add_block(did, pubkey);
    }

    async fn get_blockchain(
        State(blockchain): State<Arc<RwLock<Blockchain>>>,
    ) -> impl IntoResponse {
        let blockchain = blockchain.read().unwrap();
        let blockchain_json = serde_json::to_value(&*blockchain).unwrap();

        Json(blockchain_json) // Return JSON response
    }

}
