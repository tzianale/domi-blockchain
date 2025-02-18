use rand::Rng;

pub struct Auth {}

impl Auth{

    pub fn generate_challenge() -> String {
        let nonce: u64 = rand::thread_rng().gen();
        let challenge = format!("Login Challenge: {}", nonce);
        challenge
    }

    pub fn authenticate_user(){

    }

    pub fn is_authenticated() {

    }

    
    
}