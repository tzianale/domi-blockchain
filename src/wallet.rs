use hyper::{Body, Method, Request};
use hyper::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::fs;

use crate::crypto::Crypto;

// Separate struct used only for file serialization — includes the private key.
#[derive(Serialize, Deserialize)]
struct WalletFile {
    did: String,
    public_key_hex: String,
    private_key_hex: String,
}

pub struct Wallet {
    pub did: String,
    pub public_key_hex: String,
    private_key_hex: String,
}

impl Wallet {
    pub fn new() -> Self {
        let (private_key_hex, public_key_hex) = Crypto::generate_key_pair();
        let did = Crypto::derive_did(&public_key_hex);
        println!("DID: {}", did);
        Wallet { did, public_key_hex, private_key_hex }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let wallet_file = WalletFile {
            did: self.did.clone(),
            public_key_hex: self.public_key_hex.clone(),
            private_key_hex: self.private_key_hex.clone(),
        };
        let json = serde_json::to_string_pretty(&wallet_file)?;
        fs::write(path, json)?;
        println!("Wallet saved to {}", path);
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let json = fs::read_to_string(path)?;
        let wallet_file: WalletFile = serde_json::from_str(&json)?;
        println!("Wallet loaded from {}", path);
        Ok(Wallet {
            did: wallet_file.did,
            public_key_hex: wallet_file.public_key_hex,
            private_key_hex: wallet_file.private_key_hex,
        })
    }

    pub async fn register(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let uri = "http://127.0.0.1:3000/did/new".parse::<hyper::Uri>()?;

        let payload = json!({
            "did": self.did,
            "pubkey": self.public_key_hex
        })
        .to_string();

        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(payload))?;

        let client = Client::new();
        let response = client.request(req).await?;
        let bytes = hyper::body::to_bytes(response.into_body()).await?;
        let body: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Invalid response from server: {}", e))?;

        if let Some(err) = body["error"].as_str() {
            return Err(err.to_string().into());
        }

        Ok(())
    }

    pub async fn sign_message(&self, challenge: String) -> Option<String> {
        Crypto::sign_message(&self.private_key_hex, &challenge)
    }
}
