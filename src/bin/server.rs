use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use blockchain::Blockchain;
use domi_blockchain::blockchain;
use domi_blockchain::crypto::Crypto;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct AppState {
    blockchain: RwLock<Blockchain>,
    challenges: RwLock<HashMap<String, String>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        blockchain: RwLock::new(Blockchain::new()),
        challenges: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/secret/:did", get(get_secret))
        .route("/pubkey/:did", get(get_pubkey))
        .route("/did/new", post(add_block))
        .route("/blockchain", get(get_blockchain))
        .route("/verify", post(verify_signature))
        .with_state(state);

    let addr = "0.0.0.0:3000".parse().unwrap();
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_secret(
    State(state): State<Arc<AppState>>,
    Path(did): Path<String>,
) -> impl IntoResponse {
    let challenge = state.blockchain.read().unwrap().generate_challenge(did.clone());
    state.challenges.write().unwrap().insert(did, challenge.clone());
    challenge
}

async fn get_pubkey(
    State(state): State<Arc<AppState>>,
    Path(did): Path<String>,
) -> String {
    state.blockchain.read().unwrap().get_latest_public_key(did)
}

async fn add_block(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let did = payload["did"].as_str().unwrap_or("").to_string();
    let pubkey = payload["pubkey"].as_str().unwrap_or("").to_string();

    if did.is_empty() || pubkey.is_empty() {
        return Json(json!({ "error": "Missing did or pubkey" }));
    }

    let mut blockchain = state.blockchain.write().unwrap();

    if blockchain.did_exists(&did) {
        return Json(json!({ "error": format!("DID '{}' is already registered", did) }));
    }

    blockchain.add_block(did, pubkey);
    Json(json!({ "status": "ok" }))
}

async fn get_blockchain(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let blockchain_json = serde_json::to_value(&*state.blockchain.read().unwrap()).unwrap();
    Json(blockchain_json)
}

#[derive(Deserialize)]
struct VerifyPayload {
    did: String,
    signature: String,
}

async fn verify_signature(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyPayload>,
) -> impl IntoResponse {
    let challenge = state.challenges.read().unwrap().get(&payload.did).cloned();

    let Some(challenge) = challenge else {
        return Json(json!({ "verified": false, "error": "No challenge found for this DID" }));
    };

    let public_key = state.blockchain.read().unwrap().get_latest_public_key(payload.did.clone());

    if public_key.is_empty() {
        return Json(json!({ "verified": false, "error": "DID not found on chain" }));
    }

    let verified = Crypto::verify_signature(&public_key, &challenge, &payload.signature);

    if verified {
        // Consume the challenge so it can't be reused
        state.challenges.write().unwrap().remove(&payload.did);
    }

    Json(json!({ "verified": verified }))
}
