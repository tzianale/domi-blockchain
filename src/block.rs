use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub did: String,
    pub previous_hash: String,
    pub hash: String,
    pub public_key: String,
}

impl Block {
    pub fn new(index: u64, did: String, previous_hash: String,  public_key: String  ) -> Block {
        let timestamp = Utc::now().timestamp();
        let blockdata = format!("{}{}{}{}", index, timestamp, did, previous_hash);
        let mut hasher = Sha256::new();
        hasher.update(blockdata);
        let hash = format!("{:x}", hasher.finalize());

        Block {
            index,
            timestamp,
            did,
            previous_hash,
            hash,
            public_key,
        }
    }
}
