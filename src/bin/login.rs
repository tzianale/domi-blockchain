use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use hyper;

use domi_blockchain::wallet::Wallet;

#[derive(Deserialize)]
struct LoginForm {
    user: String,
    password: String,
}

#[derive(Deserialize)]
struct WalletQuery {
    did: String,
}


struct AppState {
    wallet: Mutex<Option<Wallet>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        wallet: Mutex::new(None),
    });

    let app = Router::new()
        .route("/login", get(login_page).post(handle_login))
        .route("/login-domi", get(login_with_domi))
        // Popup: initial wallet creation UI.
        .route("/wallet-ui", get(wallet_creation_ui))
        // Popup: sign challenge screen for the given DID.
        .route("/wallet/:did", get(wallet_sign_ui))
        .route("/wallet/add", get(wallet_add))
        .route("/wallet/sign", get(wallet_sign))
        // After login, redirect to a page showing successful login.
        .route("/:did", get(user_logged_in_page))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    println!("Login server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn login_page() -> Html<&'static str> {
    Html(
        r##"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Login</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    margin: 0;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    background-color: #f0f0f0;
                }
                .box {
                    width: 300px;
                    padding: 20px;
                    background-color: #fff;
                    border-radius: 8px;
                    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
                    text-align: center;
                }
                form {
                    margin: 0;
                }
                label {
                    display: block;
                    margin-top: 10px;
                    text-align: left;
                }
                input {
                    width: 100%;
                    padding: 8px;
                    margin-top: 5px;
                    box-sizing: border-box;
                }
                button, .domi-button {
                    display: block;
                    width: 100%;
                    margin-top: 15px;
                    padding: 10px;
                    cursor: pointer;
                    background-color: #eee;
                    border: 1px solid #ccc;
                    border-radius: 4px;
                    text-decoration: none;
                    color: #333;
                    text-align: center;
                    font-size: 16px;
                    box-sizing: border-box;
                }
            </style>
            <script>
                function openWallet() {
                    // Open the wallet creation popup.
                    window.open('/wallet-ui', 'walletPopup', 'width=400,height=600');
                }
            </script>
        </head>
        <body>
            <div class="box">
                <form method="POST" action="/login">
                    <h2>Login</h2>
                    <label for="user">User:</label>
                    <input type="text" id="user" name="user" required>
                    
                    <label for="password">Password:</label>
                    <input type="password" id="password" name="password" required>
                    
                    <button type="submit">Login</button>
                </form>
                <hr/>
                <div>
                    <h3>Login with Domi</h3>
                    <button class="domi-button" onclick="openWallet()">Login with Domi</button>
                </div>
                <div id="loginStatus" style="margin-top: 20px; font-weight: bold;"></div>
            </div>
        </body>
        </html>
        "##
    )
}

async fn handle_login(Form(input): Form<LoginForm>) -> impl IntoResponse {
    format!("Login attempted with user: {}", input.user)
}

async fn login_with_domi() -> impl IntoResponse {
    "Domi login called"
}

async fn wallet_creation_ui() -> Html<&'static str> {
    Html(
        r##"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Add Wallet</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    background-color: #f9f9f9;
                    margin: 0;
                    padding: 20px;
                }
                .container {
                    max-width: 300px;
                    margin: auto;
                    padding: 20px;
                    background-color: #fff;
                    border: 1px solid #ccc;
                    border-radius: 8px;
                    text-align: center;
                }
                input, button {
                    margin-top: 15px;
                    padding: 10px;
                    width: 100%;
                    font-size: 16px;
                }
                button {
                    border: none;
                    background-color: #eee;
                    border-radius: 4px;
                    cursor: pointer;
                }
            </style>
            <script>
                async function addWallet() {
                    const did = document.getElementById('did').value;
                    if (!did) {
                        alert("Please enter your DID");
                        return;
                    }
                    const response = await fetch('/wallet/add?did=' + encodeURIComponent(did));
                    const data = await response.json();
                    if (data.status) {
                        // Redirect to the sign challenge page for the given DID.
                        window.location.href = '/wallet/' + encodeURIComponent(did);
                    } else {
                        document.getElementById('result').innerText = 'Error: ' + (data.error || 'Unknown error');
                    }
                }
            </script>
        </head>
        <body>
            <div class="container">
                <h2>Add Wallet</h2>
                <p>Enter your wallet name (DID):</p>
                <input type="text" id="did" placeholder="e.g., did:domi:yourname">
                <button onclick="addWallet()">Add Wallet</button>
                <div id="result"></div>
            </div>
        </body>
        </html>
        "##
    )
}


async fn wallet_sign_ui(Path(did): Path<String>) -> Html<String> {
    // Fetch the challenge from the blockchain server synchronously.
    let challenge_url = format!("http://127.0.0.1:3000/secret/{}", did);
    let client = hyper::Client::new();
    let req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(challenge_url)
        .body(hyper::Body::empty())
        .expect("Failed to build request");
    let challenge = match client.request(req).await {
        Ok(resp) => match hyper::body::to_bytes(resp.into_body()).await {
            Ok(bytes) => {
                String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| "Invalid challenge".to_string())
            }
            Err(_) => "Failed to read challenge".to_string(),
        },
        Err(_) => "Failed to get challenge".to_string(),
    };

    let html = format!(
        r##"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Sign Challenge</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    background-color: #f9f9f9;
                    margin: 0;
                    padding: 20px;
                }}
                .container {{
                    max-width: 300px;
                    margin: auto;
                    padding: 20px;
                    background-color: #fff;
                    border: 1px solid #ccc;
                    border-radius: 8px;
                    text-align: center;
                }}
                button {{
                    margin-top: 15px;
                    padding: 10px;
                    width: 100%;
                    font-size: 16px;
                    border: none;
                    background-color: #eee;
                    border-radius: 4px;
                    cursor: pointer;
                }}
            </style>
            <script>
                async function signChallenge() {{
                    const response = await fetch('/wallet/sign');
                    const data = await response.json();
                    if (data.signature && data.did) {{
                        // Redirect the parent window to the logged-in page.
                        if (window.opener) {{
                            window.opener.location.href = '/' + encodeURIComponent(data.did);
                        }}
                        window.close();
                    }} else {{
                        document.getElementById('result').innerText = 'Error: ' + (data.error || 'Unknown error');
                    }}
                }}
            </script>
        </head>
        <body>
            <div class="container">
                <h2>Sign Challenge</h2>
                <p id="challenge">{challenge}</p>
                <button onclick="signChallenge()">Sign</button>
                <div id="result"></div>
            </div>
        </body>
        </html>
        "##
    );
    Html(html)
}


async fn wallet_add(
    State(state): State<Arc<AppState>>,
    Query(params): Query<WalletQuery>,
) -> impl IntoResponse {
    let new_wallet = Wallet::new(params.did);
    match new_wallet.register().await {
        Ok(_) => {
            let mut wallet_guard = state.wallet.lock().await;
            *wallet_guard = Some(new_wallet);
            Json(json!({"status": "Wallet added successfully"}))
        }
        Err(e) => Json(json!({"error": e.to_string()})),
    }
}


async fn wallet_sign(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let wallet_guard = state.wallet.lock().await;
    if let Some(ref wallet) = *wallet_guard {
        let did = &wallet.did;
        let challenge_url = format!("http://127.0.0.1:3000/secret/{}", did);
        let client = hyper::Client::new();
        let req = hyper::Request::builder()
            .method(hyper::Method::GET)
            .uri(challenge_url)
            .body(hyper::Body::empty())
            .expect("Failed to build request");
        match client.request(req).await {
            Ok(resp) => {
                match hyper::body::to_bytes(resp.into_body()).await {
                    Ok(challenge_bytes) => {
                        let challenge = String::from_utf8(challenge_bytes.to_vec()).unwrap_or_default();
                        println!("Challenge fetched in wallet_sign: {}", challenge);
                        match wallet.sign_message(challenge).await {
                            Some(signature) => Json(json!({ "signature": signature, "did": wallet.did })),
                            None => Json(json!({ "error": "Failed to sign message" })),
                        }
                    }
                    Err(e) => Json(json!({ "error": format!("Failed to read challenge: {}", e) })),
                }
            }
            Err(e) => Json(json!({ "error": format!("Failed to get challenge: {}", e) })),
        }
    } else {
        Json(json!({ "error": "No wallet added yet" }))
    }
}

async fn user_logged_in_page(Path(did): Path<String>) -> Html<String> {
    let html = format!(
        r##"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>Logged In</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    background-color: #f0f0f0;
                }}
                .message {{
                    background-color: #fff;
                    padding: 20px;
                    border-radius: 8px;
                    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
                    text-align: center;
                }}
            </style>
        </head>
        <body>
            <div class="message">
                <h2>{did} successfully logged in!</h2>
            </div>
        </body>
        </html>
        "##
    );
    Html(html)
}
