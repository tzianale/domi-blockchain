mod block;
mod blockchain;

use blockchain::Blockchain;
use std::io;

fn main() {
    // Step 1: Generate key pair
    let (private_key, public_key) = Blockchain::generate_key_pair();

    // Step 2: Generate a challenge message
    let challenge = Blockchain::generate_challenge();

    // Step 3: Sign the challenge message
    let signature = Blockchain::sign_message(&private_key, &challenge);

    println!("\n🔑 Key Pair:");
    println!("Private Key: {}", private_key);
    println!("Public Key: {}", public_key);
    
    println!("\n🖊️ Signed Challenge:");
    if let Some(sig) = signature {
        println!("Signature: {}", sig);
    }
}
