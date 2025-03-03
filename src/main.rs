mod auth;
mod block;
mod blockchain;
mod crypto;
use axum::{extract::{Path, State}, routing::{get, post}, Router};
use block::Block;
use blockchain::Blockchain;
use std::sync::{Arc, RwLock };

#[tokio::main]
async fn main() {

    let blockchain = Arc::new(RwLock::new(Blockchain::new()));

    blockchain.write().unwrap().add_block("did:alexis".to_string(), "10011999".to_string());

    let app = Router::new()
        .route("/secret", get(post_foo))
        .route("/pubkey/:did", get(get_pubkey)).with_state(blockchain);


    let addr = "0.0.0.0:3000".parse().unwrap();
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();


    async fn post_foo() -> String {
        "Hello, I'm POST /foo!".to_string()
    }

    async fn get_pubkey(State(blockchain): State<Arc<RwLock<Blockchain>>>, Path(did): Path<String>) -> String {
       blockchain.read().unwrap().get_latest_public_key(did).to_string()
    }
}
