use crate::block::Block;
use std::fs::File;
use std::io::{Read, Write};

use hex;
use k256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::Rng;
use serde::Serialize;
use serde_json;

#[derive(Clone, Serialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let index: u64 = 0;
        let did: String = "did:domi:genesis".to_string();
        let previous_hash: String = "GENESIS BLOCK".to_owned();

        Blockchain {
            chain: vec![Block::new(index, did, previous_hash, "0".to_string())],
        }
    }

    pub fn add_block(&mut self, did: String, public_key: String) {
        if let Some(last) = self.last_block() {
            let new_block = Block::new(last.index + 1, did, last.hash.clone(), public_key);

            println!("New block added with DID: {:?}", new_block.did);
            self.chain.push(new_block);
        }
    }

    pub fn last_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            if current.previous_hash != previous.hash {
                return false;
            }

            if current.index != previous.index + 1 {
                return false;
            }
        }
        true
    }

    pub fn get_latest_public_key(&self, did: String) -> String {
        for block in self.chain.iter().rev() {
            if block.did == did {
                println!("Latest public key for DID {}: {}", did, block.public_key);
                return block.public_key.to_string()
            }
        }
        println!("DID {} not found in the blockchain", did);
        "".to_string()
    }

    pub fn generate_challenge(&self, did: String) -> String {
        let nonce: u64 = rand::thread_rng().gen();
        let challenge = format!("Login Challenge: {}:{}", did, nonce);
        return challenge.to_string()
    }

    pub fn save_to_file(&self, filename: &str) {
        let json = serde_json::to_string(&self.chain).expect("Failed to serialize blockchain");
        let mut file = File::create(filename).expect("Failed to create file");
        file.write_all(json.as_bytes())
            .expect("Failed to write to file");
    }

    pub fn load_from_file(filename: &str) -> Blockchain {
        let mut file = File::open(filename).unwrap_or_else(|_| File::create(filename).unwrap());
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");

        let chain: Vec<Block> = serde_json::from_str(&contents).unwrap_or_else(|_| vec![]);
        Blockchain { chain }
    }
}
