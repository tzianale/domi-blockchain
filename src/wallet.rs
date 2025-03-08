use hyper::{Body, Method, Request};
use hyper::Client;
use serde_json::json;
use std::error::Error;

use crate::crypto::Crypto;

pub struct Wallet {
    pub did: String,
    pub public_key_hex: String,
    private_key_hex: String,
}

impl Wallet {
    pub fn new(did: String) -> Self {
        let (private_key_hex, public_key_hex) = Crypto::generate_key_pair();

        Wallet {
            did,
            public_key_hex,
            private_key_hex,
        }
    }

    pub async fn register(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let uri = "http://127.0.0.1:3000/did/new".parse::<hyper::Uri>()?;

        let payload = json!({
            "did": self.did,
            "pubkey": self.public_key_hex
        })
        .to_string();

        println!("Sending JSON Payload: {}", payload); 

        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(Body::from(payload))?;

        let client = Client::new();
        let response = client.request(req).await?;

        println!("Response Status: {}", response.status());

        Ok(())
    }

    pub async fn sign_message(&self , challange : String ) -> Option<String> {
        Crypto::sign_message(&self.private_key_hex, &challange)
    }

}
