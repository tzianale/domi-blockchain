use crate::crypto::{self, Crypto};


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




}
