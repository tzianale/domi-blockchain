use crate::block::Block;
use std::fs::File;
use std::io::{Read, Write};

use rand::Rng;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Clone, Serialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            chain: vec![Block::new(
                0,
                "did:domi:genesis".to_string(),
                "GENESIS BLOCK".to_string(),
                "0".to_string(),
            )],
        }
    }

    pub fn did_exists(&self, did: &str) -> bool {
        self.chain.iter().any(|block| block.did == did)
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

            // Check the chain link
            if current.previous_hash != previous.hash {
                return false;
            }

            if current.index != previous.index + 1 {
                return false;
            }

            // Recompute the hash and verify it matches the stored value
            let expected_hash = Self::compute_hash(
                current.index,
                current.timestamp,
                &current.did,
                &current.previous_hash,
                &current.public_key,
            );
            if current.hash != expected_hash {
                return false;
            }
        }
        true
    }

    pub fn compute_hash(
        index: u64,
        timestamp: i64,
        did: &str,
        previous_hash: &str,
        public_key: &str,
    ) -> String {
        let data = format!("{}{}{}{}{}", index, timestamp, did, previous_hash, public_key);
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    pub fn get_latest_public_key(&self, did: String) -> String {
        for block in self.chain.iter().rev() {
            if block.did == did {
                return block.public_key.clone();
            }
        }
        "".to_string()
    }

    pub fn generate_challenge(&self, did: String) -> String {
        let nonce: u64 = rand::thread_rng().gen();
        format!("{}:{}", did, nonce)
    }

    pub fn save_to_file(&self, filename: &str) {
        let json = serde_json::to_string(&self.chain).expect("Failed to serialize blockchain");
        let mut file = File::create(filename).expect("Failed to create file");
        file.write_all(json.as_bytes()).expect("Failed to write to file");
    }

    pub fn load_from_file(filename: &str) -> Blockchain {
        let mut file = File::open(filename).unwrap_or_else(|_| File::create(filename).unwrap());
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read file");

        let chain: Vec<Block> = serde_json::from_str(&contents).unwrap_or_else(|_| vec![]);
        Blockchain { chain }
    }
}
