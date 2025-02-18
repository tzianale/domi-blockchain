// main.rs

// Import the external modules which provide authentication, blockchain, and crypto functionality.
mod auth;
mod block;
mod blockchain;
mod crypto;

use eframe::App;
use eframe::egui;
use egui::{CentralPanel, TopBottomPanel, Window, Frame};


// Our application state which holds UI state, login credentials, wallet data, and the blockchain.
struct MyApp {
    // Traditional login fields
    username: String,
    password: String,
    login_feedback: String,
    
    // Wallet modal state and input field for DID
    show_wallet_modal: bool,
    wallet_did_input: String,
    
    // Wallet connection status and keys
    wallet_connected: bool,
    wallet_did: Option<String>,
    wallet_private_key: Option<String>,
    wallet_public_key: Option<String>,
    
    // The blockchain to which wallet connection events are recorded
    blockchain: blockchain::Blockchain,
    
    // DOMI Sign-In modal state and challenge
    show_domi_modal: bool,
    domi_challenge: Option<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            login_feedback: String::new(),
            show_wallet_modal: false,
            wallet_did_input: String::new(),
            wallet_connected: false,
            wallet_did: None,
            wallet_private_key: None,
            wallet_public_key: None,
            blockchain: blockchain::Blockchain::new(),
            show_domi_modal: false,
            domi_challenge: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top header panel with the website title on the left and a 'D' icon on the right.
        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Demo Website");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    // Clicking the 'D' icon toggles the wallet modal.
                    if ui.button("D").clicked() {
                        self.show_wallet_modal = true;
                    }
                });
            });
        });

        // Central panel for the traditional login form.
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Login");
            ui.separator();
            ui.label("Username:");
            ui.text_edit_singleline(&mut self.username);
            ui.label("Password:");
            ui.add(egui::TextEdit::singleline(&mut self.password).password(true));

            ui.horizontal(|ui| {
                // Traditional login button: simply checks that both fields are non-empty.
                if ui.button("Login").clicked() {
                    if self.username.trim().is_empty() || self.password.trim().is_empty() {
                        self.login_feedback = "Please fill in both username and password.".to_owned();
                    } else {
                        self.login_feedback = format!("Welcome, {}", self.username);
                    }
                }

                // Trigger the DOMI sign-in process.
                if ui.button("Sign in with DOMI").clicked() {
                    if !self.wallet_connected {
                        self.login_feedback =
                            "Please connect your wallet first via the 'D' icon.".to_owned();
                    } else {
                        // Generate a random challenge using the auth module.
                        self.domi_challenge = Some(auth::Auth::generate_challenge());
                        self.show_domi_modal = true;
                    }
                }
            });
            ui.label(&self.login_feedback);
        });

        // Wallet Modal (triggered by the 'D' icon in the header).
        if self.show_wallet_modal {
            Window::new("Wallet")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    // If the wallet is not connected, show a form to enter a DID and connect the wallet.
                    if !self.wallet_connected {
                        ui.label("Enter your Decentralized Identifier (DID):");
                        ui.text_edit_singleline(&mut self.wallet_did_input);
                        if ui.button("Connect Wallet").clicked() {
                            if self.wallet_did_input.trim().is_empty() {
                                self.login_feedback = "DID cannot be empty.".to_owned();
                            } else {
                                // Generate an ECDSA key pair using the crypto module.
                                let (priv_key, pub_key) = crypto::Crypto::generate_key_pair();
                                self.wallet_private_key = Some(priv_key);
                                self.wallet_public_key = Some(pub_key.clone());
                                self.wallet_did = Some(self.wallet_did_input.clone());
                                // Record the connection by adding a new block to the blockchain.
                                self.blockchain.add_block(self.wallet_did_input.clone(), pub_key);
                                self.wallet_connected = true;
                                self.login_feedback = "Wallet connected successfully.".to_owned();
                                self.wallet_did_input.clear();
                            }
                        }
                    } else {
                        // If the wallet is connected, display the wallet details and blockchain history.
                        ui.label("Wallet is connected.");
                        if let Some(did) = &self.wallet_did {
                            ui.label(format!("DID: {}", did));
                        }
                        if let Some(pub_key) = &self.wallet_public_key {
                            ui.label(format!("Public Key: {}", pub_key));
                        }
                        ui.separator();
                        ui.label("Blockchain Blocks:");
                        for block in &self.blockchain.chain {
                            ui.label(format!(
                                "Block {}: DID: {}, Hash: {}",
                                block.index, block.did, block.hash
                            ));
                        }
                        if ui.button("Disconnect Wallet").clicked() {
                            // Reset all wallet and blockchain data.
                            self.wallet_connected = false;
                            self.wallet_did = None;
                            self.wallet_private_key = None;
                            self.wallet_public_key = None;
                            self.blockchain = blockchain::Blockchain::new();
                            self.login_feedback = "Wallet disconnected.".to_owned();
                        }
                    }
                    if ui.button("Close").clicked() {
                        self.show_wallet_modal = false;
                    }
                });
        }

        // DOMI Sign-In Modal (triggered by the "Sign in with DOMI" button).
        if self.show_domi_modal {
            Window::new("DOMI Sign-In")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(challenge) = &self.domi_challenge {
                        ui.label(format!("Challenge: {}", challenge));
                        if ui.button("Sign Challenge").clicked() {
                            // Ensure wallet keys are available.
                            if let (Some(priv_key), Some(pub_key)) =
                                (&self.wallet_private_key, &self.wallet_public_key)
                            {
                                // Sign the challenge using the wallet’s private key.
                                if let Some(signature) =
                                    crypto::Crypto::sign_message(priv_key, challenge)
                                {
                                    // Verify the signature using the wallet’s public key.
                                    let verified =
                                        crypto::Crypto::verify_signature(pub_key, challenge, &signature);
                                    if verified {
                                        if let Some(did) = &self.wallet_did {
                                            self.login_feedback =
                                                format!("Welcome, {} (DOMI)", did);
                                        } else {
                                            self.login_feedback = "Wallet DID not found.".to_owned();
                                        }
                                        self.show_domi_modal = false;
                                    } else {
                                        self.login_feedback = "Signature verification failed.".to_owned();
                                    }
                                } else {
                                    self.login_feedback = "Signing failed.".to_owned();
                                }
                            } else {
                                self.login_feedback = "Wallet keys not available.".to_owned();
                            }
                        }
                    } else {
                        ui.label("No challenge generated.");
                    }
                    if ui.button("Close").clicked() {
                        self.show_domi_modal = false;
                    }
                });
        }
    }
}

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.decorated = true;
    if let Err(e) = eframe::run_native(
        "Demo Website",
        native_options,
        Box::new(|_cc| Box::new(MyApp::default())),
    ) {
        eprintln!("Error: {}", e);
    }
}
