use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use hyper;

use domi_blockchain::wallet::Wallet;

const WALLET_FILE: &str = "wallet.json";

struct AppState {
    wallet: Mutex<Option<Wallet>>,
    challenge: Mutex<Option<String>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        wallet: Mutex::new(None),
        challenge: Mutex::new(None),
    });

    let app = Router::new()
        .route("/login", get(login_page))
        .route("/account/create", get(create_account_ui))
        .route("/account/new", get(create_account))
        .route("/account/load", get(load_account))
        .route("/wallet-ui", get(wallet_creation_ui))
        .route("/wallet/sign", get(wallet_sign))
        .route("/wallet/:did", get(wallet_sign_ui))
        .route("/:did", get(user_logged_in_page))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    println!("Login server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Shared base styles injected into every page
fn base_styles() -> &'static str {
    r#"
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
        *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

        body {
            font-family: 'Inter', sans-serif;
            background: #0d0d14;
            color: #e2e2f0;
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
        }

        .card {
            background: #16161f;
            border: 1px solid #2a2a3d;
            border-radius: 16px;
            padding: 40px;
            width: 380px;
            box-shadow: 0 24px 64px rgba(0,0,0,0.4);
        }

        .logo {
            font-size: 13px;
            font-weight: 600;
            letter-spacing: 0.15em;
            text-transform: uppercase;
            color: #7b6fff;
            margin-bottom: 32px;
        }

        h1 {
            font-size: 22px;
            font-weight: 600;
            color: #fff;
            margin-bottom: 8px;
        }

        .subtitle {
            font-size: 14px;
            color: #6b6b8a;
            margin-bottom: 28px;
            line-height: 1.5;
        }

        .btn {
            display: block;
            width: 100%;
            padding: 13px 20px;
            font-family: 'Inter', sans-serif;
            font-size: 14px;
            font-weight: 500;
            border: none;
            border-radius: 10px;
            cursor: pointer;
            text-align: center;
            text-decoration: none;
            transition: opacity 0.15s, transform 0.1s;
        }

        .btn:active { transform: scale(0.98); }

        .btn-primary {
            background: linear-gradient(135deg, #7b6fff, #5b4fcf);
            color: #fff;
        }

        .btn-primary:hover { opacity: 0.9; }

        .btn-secondary {
            background: #1f1f2e;
            color: #a0a0c0;
            border: 1px solid #2a2a3d;
            margin-top: 10px;
        }

        .btn-secondary:hover { background: #252538; color: #fff; }

        .divider {
            display: flex;
            align-items: center;
            gap: 12px;
            margin: 24px 0;
            color: #2a2a3d;
            font-size: 12px;
        }

        .divider::before, .divider::after {
            content: '';
            flex: 1;
            height: 1px;
            background: #2a2a3d;
        }

        .mono {
            font-family: 'JetBrains Mono', monospace;
            font-size: 11px;
            word-break: break-all;
            background: #0d0d14;
            border: 1px solid #2a2a3d;
            border-radius: 8px;
            padding: 12px;
            color: #7b6fff;
            line-height: 1.6;
            margin-top: 16px;
        }

        .status {
            margin-top: 16px;
            font-size: 13px;
            min-height: 20px;
            color: #6b6b8a;
        }

        .status.error { color: #ff6b6b; }
        .status.success { color: #6bffb8; }

        .tag {
            display: inline-block;
            font-size: 11px;
            font-weight: 500;
            padding: 3px 8px;
            border-radius: 4px;
            background: #1f1f2e;
            border: 1px solid #2a2a3d;
            color: #6b6b8a;
            margin-bottom: 20px;
        }

        a { color: #7b6fff; text-decoration: none; }
        a:hover { text-decoration: underline; }

        .spinner {
            display: inline-block;
            width: 14px;
            height: 14px;
            border: 2px solid rgba(255,255,255,0.2);
            border-top-color: #fff;
            border-radius: 50%;
            animation: spin 0.6s linear infinite;
            vertical-align: middle;
            margin-right: 6px;
        }

        @keyframes spin { to { transform: rotate(360deg); } }
    </style>
    "#
}

async fn login_page() -> Html<&'static str> {
    Html(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Domi — Login</title>
    <!-- base styles injected below -->
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
        *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: 'Inter', sans-serif; background: #0d0d14; color: #e2e2f0; min-height: 100vh; display: flex; justify-content: center; align-items: center; }
        .card { background: #16161f; border: 1px solid #2a2a3d; border-radius: 16px; padding: 40px; width: 380px; box-shadow: 0 24px 64px rgba(0,0,0,0.4); }
        .logo { font-size: 12px; font-weight: 600; letter-spacing: 0.15em; text-transform: uppercase; color: #7b6fff; margin-bottom: 32px; }
        h1 { font-size: 22px; font-weight: 600; color: #fff; margin-bottom: 8px; }
        .subtitle { font-size: 14px; color: #6b6b8a; margin-bottom: 28px; line-height: 1.5; }
        .btn { display: block; width: 100%; padding: 13px 20px; font-family: 'Inter', sans-serif; font-size: 14px; font-weight: 500; border: none; border-radius: 10px; cursor: pointer; text-align: center; text-decoration: none; transition: opacity 0.15s, transform 0.1s; }
        .btn:active { transform: scale(0.98); }
        .btn-primary { background: linear-gradient(135deg, #7b6fff, #5b4fcf); color: #fff; }
        .btn-primary:hover { opacity: 0.9; }
        .btn-secondary { background: #1f1f2e; color: #a0a0c0; border: 1px solid #2a2a3d; margin-top: 10px; }
        .btn-secondary:hover { background: #252538; color: #fff; }
        .divider { display: flex; align-items: center; gap: 12px; margin: 24px 0; color: #2a2a3d; font-size: 12px; }
        .divider::before, .divider::after { content: ''; flex: 1; height: 1px; background: #2a2a3d; }
        a { color: #7b6fff; text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
    <script>
        function openWallet() {
            window.open('/wallet-ui', 'walletPopup', 'width=420,height=500');
        }
    </script>
</head>
<body>
    <div class="card">
        <div class="logo">Domi Identity</div>
        <h1>Welcome back</h1>
        <p class="subtitle">Sign in using your cryptographic identity. No password needed.</p>
        <button class="btn btn-primary" onclick="openWallet()">Sign in with Domi</button>
        <div class="divider">or</div>
        <a class="btn btn-secondary" href="/account/create">Create a new account</a>
    </div>
</body>
</html>"##
    )
}

async fn create_account_ui() -> Html<&'static str> {
    Html(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Domi — Create Account</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
        *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: 'Inter', sans-serif; background: #0d0d14; color: #e2e2f0; min-height: 100vh; display: flex; justify-content: center; align-items: center; }
        .card { background: #16161f; border: 1px solid #2a2a3d; border-radius: 16px; padding: 40px; width: 420px; box-shadow: 0 24px 64px rgba(0,0,0,0.4); }
        .logo { font-size: 12px; font-weight: 600; letter-spacing: 0.15em; text-transform: uppercase; color: #7b6fff; margin-bottom: 32px; }
        h1 { font-size: 22px; font-weight: 600; color: #fff; margin-bottom: 8px; }
        .subtitle { font-size: 14px; color: #6b6b8a; margin-bottom: 28px; line-height: 1.5; }
        .btn { display: block; width: 100%; padding: 13px 20px; font-family: 'Inter', sans-serif; font-size: 14px; font-weight: 500; border: none; border-radius: 10px; cursor: pointer; text-align: center; text-decoration: none; transition: opacity 0.15s, transform 0.1s; }
        .btn:active { transform: scale(0.98); }
        .btn-primary { background: linear-gradient(135deg, #7b6fff, #5b4fcf); color: #fff; }
        .btn-primary:hover { opacity: 0.9; }
        .mono { font-family: 'JetBrains Mono', monospace; font-size: 11px; word-break: break-all; background: #0d0d14; border: 1px solid #2a2a3d; border-radius: 8px; padding: 12px; color: #7b6fff; line-height: 1.6; margin-top: 16px; }
        .status { margin-top: 16px; font-size: 13px; color: #6b6b8a; }
        .status.error { color: #ff6b6b; }
        .info-box { margin-top: 20px; background: #1a1a28; border: 1px solid #2a2a3d; border-radius: 10px; padding: 16px; font-size: 13px; color: #6b6b8a; line-height: 1.6; }
        .info-box strong { color: #a0a0c0; }
        a { color: #7b6fff; text-decoration: none; }
        .spinner { display: inline-block; width: 14px; height: 14px; border: 2px solid rgba(255,255,255,0.2); border-top-color: #fff; border-radius: 50%; animation: spin 0.6s linear infinite; vertical-align: middle; margin-right: 6px; }
        @keyframes spin { to { transform: rotate(360deg); } }
    </style>
    <script>
        async function createAccount() {
            const btn = document.getElementById('btn');
            const status = document.getElementById('status');
            btn.disabled = true;
            btn.innerHTML = '<span class="spinner"></span>Generating identity...';
            status.className = 'status';
            status.innerHTML = '';

            const response = await fetch('/account/new');
            const data = await response.json();

            if (data.status) {
                btn.style.display = 'none';
                status.className = 'status success';
                status.innerHTML =
                    'Identity created and registered on chain.' +
                    '<div class="mono">' + data.did + '</div>' +
                    '<div style="margin-top:20px;font-size:13px;color:#6b6b8a;">Your wallet has been saved to <code style="color:#a0a0c0">wallet.json</code>. Keep it safe.<br><br><a href="/login">Back to login &rarr;</a></div>';
            } else {
                btn.disabled = false;
                btn.innerHTML = 'Generate Identity';
                status.className = 'status error';
                status.innerText = 'Error: ' + (data.error || 'Something went wrong');
            }
        }
    </script>
</head>
<body>
    <div class="card">
        <div class="logo">Domi Identity</div>
        <h1>Create account</h1>
        <p class="subtitle">A unique identity will be generated from your keypair. You don't choose it — the math does.</p>
        <button class="btn btn-primary" id="btn" onclick="createAccount()">Generate Identity</button>
        <div id="status" class="status"></div>
        <div class="info-box">
            <strong>How it works</strong><br>
            A keypair is generated, your DID is derived from the public key using SHA256, and then registered on the blockchain. Your private key is saved locally.
        </div>
    </div>
</body>
</html>"##
    )
}

async fn create_account(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let wallet = Wallet::new();

    if let Err(e) = wallet.save_to_file(WALLET_FILE) {
        return Json(json!({ "error": format!("Failed to save wallet: {}", e) }));
    }

    match wallet.register().await {
        Ok(_) => {
            let did = wallet.did.clone();
            *state.wallet.lock().await = Some(wallet);
            Json(json!({ "status": "Account created", "did": did }))
        }
        Err(e) => Json(json!({ "error": format!("Failed to register on blockchain: {}", e) })),
    }
}

async fn load_account(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let wallet = match Wallet::load_from_file(WALLET_FILE) {
        Ok(w) => w,
        Err(e) => return Json(json!({ "error": format!("Failed to load wallet: {}", e) })),
    };
    let did = wallet.did.clone();
    *state.wallet.lock().await = Some(wallet);
    Json(json!({ "status": "Wallet loaded", "did": did }))
}

async fn wallet_creation_ui() -> Html<&'static str> {
    Html(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Domi Wallet</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
        *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: 'Inter', sans-serif; background: #0d0d14; color: #e2e2f0; min-height: 100vh; display: flex; justify-content: center; align-items: center; }
        .card { background: #16161f; border: 1px solid #2a2a3d; border-radius: 16px; padding: 36px; width: 360px; box-shadow: 0 24px 64px rgba(0,0,0,0.4); }
        .logo { font-size: 12px; font-weight: 600; letter-spacing: 0.15em; text-transform: uppercase; color: #7b6fff; margin-bottom: 28px; }
        h1 { font-size: 20px; font-weight: 600; color: #fff; margin-bottom: 8px; }
        .subtitle { font-size: 13px; color: #6b6b8a; margin-bottom: 24px; line-height: 1.5; }
        .btn { display: block; width: 100%; padding: 13px 20px; font-family: 'Inter', sans-serif; font-size: 14px; font-weight: 500; border: none; border-radius: 10px; cursor: pointer; text-align: center; transition: opacity 0.15s, transform 0.1s; }
        .btn:active { transform: scale(0.98); }
        .btn-primary { background: linear-gradient(135deg, #7b6fff, #5b4fcf); color: #fff; }
        .btn-primary:hover { opacity: 0.9; }
        .btn:disabled { opacity: 0.5; cursor: not-allowed; }
        .status { margin-top: 14px; font-size: 13px; color: #ff6b6b; }
        .spinner { display: inline-block; width: 14px; height: 14px; border: 2px solid rgba(255,255,255,0.2); border-top-color: #fff; border-radius: 50%; animation: spin 0.6s linear infinite; vertical-align: middle; margin-right: 6px; }
        @keyframes spin { to { transform: rotate(360deg); } }
        .wallet-icon { font-size: 32px; margin-bottom: 16px; }
    </style>
    <script>
        async function loadAndSign() {
            const btn = document.getElementById('btn');
            const status = document.getElementById('status');
            btn.disabled = true;
            btn.innerHTML = '<span class="spinner"></span>Loading wallet...';
            status.innerText = '';

            const loadRes = await fetch('/account/load');
            const loadData = await loadRes.json();

            if (!loadData.status) {
                btn.disabled = false;
                btn.innerHTML = 'Use My Wallet';
                status.innerText = loadData.error || 'Could not load wallet';
                return;
            }

            window.location.href = '/wallet/' + encodeURIComponent(loadData.did);
        }
    </script>
</head>
<body>
    <div class="card">
        <div class="logo">Domi Identity</div>
        <div class="wallet-icon">🔑</div>
        <h1>Your wallet</h1>
        <p class="subtitle">Click below to load your saved identity and sign in.</p>
        <button class="btn btn-primary" id="btn" onclick="loadAndSign()">Use My Wallet</button>
        <div id="status" class="status"></div>
    </div>
</body>
</html>"##
    )
}

async fn wallet_sign_ui(
    State(state): State<Arc<AppState>>,
    Path(did): Path<String>,
) -> Html<String> {
    let challenge_url = format!("http://127.0.0.1:3000/secret/{}", did);
    let client = hyper::Client::new();
    let req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri(challenge_url)
        .body(hyper::Body::empty())
        .expect("Failed to build request");

    let challenge = match client.request(req).await {
        Ok(resp) => match hyper::body::to_bytes(resp.into_body()).await {
            Ok(bytes) => String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| "Invalid challenge".to_string()),
            Err(_) => "Failed to read challenge".to_string(),
        },
        Err(_) => "Failed to connect to blockchain server".to_string(),
    };

    *state.challenge.lock().await = Some(challenge.clone());

    let html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Domi — Sign Challenge</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
        *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: 'Inter', sans-serif; background: #0d0d14; color: #e2e2f0; min-height: 100vh; display: flex; justify-content: center; align-items: center; }}
        .card {{ background: #16161f; border: 1px solid #2a2a3d; border-radius: 16px; padding: 36px; width: 380px; box-shadow: 0 24px 64px rgba(0,0,0,0.4); }}
        .logo {{ font-size: 12px; font-weight: 600; letter-spacing: 0.15em; text-transform: uppercase; color: #7b6fff; margin-bottom: 28px; }}
        h1 {{ font-size: 20px; font-weight: 600; color: #fff; margin-bottom: 8px; }}
        .subtitle {{ font-size: 13px; color: #6b6b8a; margin-bottom: 20px; line-height: 1.5; }}
        .challenge-label {{ font-size: 11px; font-weight: 500; text-transform: uppercase; letter-spacing: 0.1em; color: #4a4a6a; margin-bottom: 8px; }}
        .mono {{ font-family: 'JetBrains Mono', monospace; font-size: 11px; word-break: break-all; background: #0d0d14; border: 1px solid #2a2a3d; border-radius: 8px; padding: 12px; color: #7b6fff; line-height: 1.6; margin-bottom: 20px; }}
        .btn {{ display: block; width: 100%; padding: 13px 20px; font-family: 'Inter', sans-serif; font-size: 14px; font-weight: 500; border: none; border-radius: 10px; cursor: pointer; text-align: center; transition: opacity 0.15s, transform 0.1s; }}
        .btn:active {{ transform: scale(0.98); }}
        .btn-primary {{ background: linear-gradient(135deg, #7b6fff, #5b4fcf); color: #fff; }}
        .btn-primary:hover {{ opacity: 0.9; }}
        .btn:disabled {{ opacity: 0.5; cursor: not-allowed; }}
        .status {{ margin-top: 14px; font-size: 13px; color: #ff6b6b; }}
        .spinner {{ display: inline-block; width: 14px; height: 14px; border: 2px solid rgba(255,255,255,0.2); border-top-color: #fff; border-radius: 50%; animation: spin 0.6s linear infinite; vertical-align: middle; margin-right: 6px; }}
        @keyframes spin {{ to {{ transform: rotate(360deg); }} }}
    </style>
    <script>
        async function signChallenge() {{
            const btn = document.getElementById('btn');
            const status = document.getElementById('status');
            btn.disabled = true;
            btn.innerHTML = '<span class="spinner"></span>Signing...';
            status.innerText = '';

            const response = await fetch('/wallet/sign');
            const data = await response.json();

            if (data.verified && data.did) {{
                btn.innerHTML = 'Verified ✓';
                if (window.opener) {{
                    window.opener.location.href = '/' + data.did;
                }}
                setTimeout(() => window.close(), 600);
            }} else {{
                btn.disabled = false;
                btn.innerHTML = 'Sign & Login';
                status.innerText = data.error || 'Verification failed';
            }}
        }}
    </script>
</head>
<body>
    <div class="card">
        <div class="logo">Domi Identity</div>
        <h1>Sign challenge</h1>
        <p class="subtitle">Signing this proves you control the private key for this identity.</p>
        <div class="challenge-label">Challenge</div>
        <div class="mono">{challenge}</div>
        <button class="btn btn-primary" id="btn" onclick="signChallenge()">Sign &amp; Login</button>
        <div id="status" class="status"></div>
    </div>
</body>
</html>"##);

    Html(html)
}

async fn wallet_sign(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    {
        let mut wallet_guard = state.wallet.lock().await;
        if wallet_guard.is_none() {
            match Wallet::load_from_file(WALLET_FILE) {
                Ok(w) => *wallet_guard = Some(w),
                Err(_) => return Json(json!({ "error": "No wallet found — create an account first" })),
            }
        }
    }

    let wallet_guard = state.wallet.lock().await;
    let Some(ref wallet) = *wallet_guard else {
        return Json(json!({ "error": "No wallet loaded" }));
    };

    let challenge_guard = state.challenge.lock().await;
    let Some(ref challenge) = *challenge_guard else {
        return Json(json!({ "error": "No challenge available" }));
    };

    let Some(signature) = wallet.sign_message(challenge.clone()).await else {
        return Json(json!({ "error": "Failed to sign challenge" }));
    };

    let verify_payload = json!({ "did": wallet.did, "signature": signature }).to_string();
    let client = hyper::Client::new();
    let req = hyper::Request::builder()
        .method(hyper::Method::POST)
        .uri("http://127.0.0.1:3000/verify")
        .header("Content-Type", "application/json")
        .body(hyper::Body::from(verify_payload))
        .expect("Failed to build verify request");

    match client.request(req).await {
        Ok(resp) => match hyper::body::to_bytes(resp.into_body()).await {
            Ok(bytes) => {
                let result: serde_json::Value = serde_json::from_slice(&bytes)
                    .map_err(|e| format!("Invalid response from server: {}", e))
                    .unwrap_or_default();
                if result["verified"].as_bool().unwrap_or(false) {
                    Json(json!({ "verified": true, "did": wallet.did }))
                } else {
                    Json(json!({ "error": result["error"].as_str().unwrap_or("Verification failed") }))
                }
            }
            Err(e) => Json(json!({ "error": format!("Failed to read verify response: {}", e) })),
        },
        Err(e) => Json(json!({ "error": format!("Failed to reach blockchain server: {}", e) })),
    }
}

async fn user_logged_in_page(Path(did): Path<String>) -> Html<String> {
    let html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Domi — Logged In</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
    <style>
        *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: 'Inter', sans-serif; background: #0d0d14; color: #e2e2f0; min-height: 100vh; display: flex; justify-content: center; align-items: center; }}
        .card {{ background: #16161f; border: 1px solid #2a2a3d; border-radius: 16px; padding: 48px 40px; width: 440px; box-shadow: 0 24px 64px rgba(0,0,0,0.4); text-align: center; }}
        .check {{ font-size: 48px; margin-bottom: 20px; }}
        h1 {{ font-size: 22px; font-weight: 600; color: #fff; margin-bottom: 8px; }}
        .subtitle {{ font-size: 14px; color: #6b6b8a; margin-bottom: 24px; }}
        .did-label {{ font-size: 11px; font-weight: 500; text-transform: uppercase; letter-spacing: 0.1em; color: #4a4a6a; margin-bottom: 8px; }}
        .mono {{ font-family: 'JetBrains Mono', monospace; font-size: 11px; word-break: break-all; background: #0d0d14; border: 1px solid #2a2a3d; border-radius: 8px; padding: 12px; color: #7b6fff; line-height: 1.6; }}
        .badge {{ display: inline-flex; align-items: center; gap: 6px; background: rgba(107,255,184,0.08); border: 1px solid rgba(107,255,184,0.2); color: #6bffb8; font-size: 12px; font-weight: 500; padding: 4px 10px; border-radius: 20px; margin-bottom: 24px; }}
    </style>
</head>
<body>
    <div class="card">
        <div class="check">✓</div>
        <span class="badge">Identity verified</span>
        <h1>You're in</h1>
        <p class="subtitle">Successfully authenticated with your Domi identity.</p>
        <div class="did-label">Your DID</div>
        <div class="mono">{did}</div>
    </div>
</body>
</html>"##);

    Html(html)
}
