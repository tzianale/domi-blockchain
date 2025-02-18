use hex;
use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature, SigningKey, VerifyingKey,
};
use rand::rngs::OsRng;

pub struct Crypto;

impl Crypto {
    pub fn sign_message(private_key_hex: &str, message: &str) -> Option<String> {
        let private_key_bytes = hex::decode(private_key_hex).expect("Invalid hex for private key");
        let private_key_array: [u8; 32] = private_key_bytes
            .try_into()
            .expect("Private key must be 32 bytes long");

        let signing_key = SigningKey::from_bytes(&private_key_array.into())
            .expect("Failed to create signing key");

        let signature: Signature = signing_key.sign(message.as_bytes());
        let signature_hex = hex::encode(signature.to_der());

        println!("🖊️ Signed Message: {}", signature_hex);
        Some(signature_hex)
    }

    pub fn generate_key_pair() -> (String, String) {
        let signing_key = SigningKey::random(&mut OsRng);
        let private_key_bytes = signing_key.to_bytes();

        let verifying_key = VerifyingKey::from(&signing_key);
        let public_key_bytes = verifying_key.to_encoded_point(false);

        let private_key_hex = hex::encode(private_key_bytes);
        let public_key_hex = hex::encode(public_key_bytes.as_bytes());

        println!("New Key Pair Generated!");
        println!("Private Key (Keep Safe!): {}", private_key_hex);
        println!("Public Key (Store on Blockchain): {}", public_key_hex);

        (private_key_hex, public_key_hex)
    }

    pub fn verify_signature(public_key: &str, message: &str, signature: &str) -> bool {
        // Convert the public key from hex string to VerifyingKey
        let public_key_bytes = match hex::decode(public_key) {
            Ok(bytes) => bytes,
            Err(_) => {
                println!("❌ Invalid public key format");
                return false;
            }
        };

        let verifying_key = match VerifyingKey::from_sec1_bytes(&public_key_bytes) {
            Ok(key) => key,
            Err(_) => {
                println!("❌ Failed to create verifying key");
                return false;
            }
        };

        // Convert the signature from hex to Signature object
        let signature_bytes = match hex::decode(signature) {
            Ok(bytes) => bytes,
            Err(_) => {
                println!("❌ Invalid signature format");
                return false;
            }
        };

        let signature = match Signature::from_der(&signature_bytes) {
            Ok(sig) => sig,
            Err(_) => {
                println!("❌ Invalid signature bytes");
                return false;
            }
        };

        // Verify the signature
        match verifying_key.verify(message.as_bytes(), &signature) {
            Ok(_) => {
                println!("✅ Signature is valid");
                true
            }
            Err(_) => {
                println!("❌ Signature verification failed");
                false
            }
        }
    }
}
