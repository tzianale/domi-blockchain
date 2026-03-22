# domi-blockchain

A prototype blockchain written in Rust for decentralized identity (DID). The idea is simple: instead of logging in with a username and password, you own a cryptographic identity that lives on a blockchain. You prove who you are by signing a challenge with your private key, no password, no central authority.

This is a prototype, not production software.

---

## What it does

- Generates a unique identity (DID) from your keypair, mathematically derived, not chosen
- Registers that identity on a local blockchain
- Lets you log into a web interface by signing a challenge with your key
- The blockchain verifies the signature against your registered public key

Your DID looks like this: `did:domi:a3f9bc12e8d4c71f...`

It's derived from your public key using SHA256, similar to how Ethereum addresses work. No two people get the same DID unless they have the same keypair, which is cryptographically impossible in practice.

---

## How it works

There are two servers:

**Blockchain server** (port 3000), handles the chain itself
- Stores DIDs and public keys as blocks
- Issues challenges for login
- Verifies signatures

**Login server** (port 4000), the web UI
- Account creation
- Wallet management
- Signs challenges and talks to the blockchain server

The login flow:
1. Create an account, a keypair is generated, your DID is derived, saved to `wallet.json`, and registered on the blockchain
2. To log in, your wallet loads from disk, the blockchain issues a challenge, you sign it, the blockchain verifies the signature against your registered public key
3. If it matches, you're in

---

## Running it

You need two terminals.

**Terminal 1, start the blockchain server:**
```bash
cargo run --bin server
```

**Terminal 2, start the login server:**
```bash
cargo run --bin login
```

Then open `http://127.0.0.1:4000/login` in your browser.

**First time:** click "Create Account" to generate your identity.
**After that:** click "Sign in with Domi" to log in.

---

## API endpoints (blockchain server)

| Method | Route | What it does |
|--------|-------|--------------|
| `POST` | `/did/new` | Register a new DID |
| `GET` | `/secret/:did` | Get a login challenge |
| `POST` | `/verify` | Verify a signed challenge |
| `GET` | `/pubkey/:did` | Get the public key for a DID |
| `GET` | `/blockchain` | See the full chain |

---

## Tech stack

- **Rust**, everything
- **axum + tokio**, async HTTP servers
- **k256**, ECDSA cryptography (secp256k1, same curve as Bitcoin/Ethereum)
- **sha2**, block hashing and DID derivation
- **serde / serde_json**, serialization

---

## Current limitations

This is a prototype. A few things are intentionally left simple:

- The blockchain lives in memory, restarting the server wipes it
- The wallet file (`wallet.json`) stores the private key in plain text
- Only one wallet per machine right now
- No HTTPS, no sessions, no rate limiting

---

## Future plans

**Persistence**
Save the blockchain to disk so it survives restarts. The code for this already exists (`save_to_file` / `load_from_file`), it just needs to be wired up.

**Verifiable Credentials**
The next big thing. The idea is to separate identity from claims, so a trusted issuer (like a government or KYC provider) can sign a credential saying "this person is over 18" and attach it to a DID. The website verifying age never learns who you are, just that the claim is valid. This is the W3C Verifiable Credentials model.

**Private key encryption**
The wallet file should be encrypted with a password before anything leaves localhost.

**Multi-account support**
Right now one machine = one wallet. Should support multiple identities.

**Browser extension**
The login popup already mimics how MetaMask works. The next step is a proper browser extension that holds your wallet and signs challenges without exposing the key to any server.

**Migrate to Solana or Ethereum**
For a public, trustless DID registry, anchoring on an existing chain makes more sense than running your own. The `did:domi:` method could be implemented as a Solana program or Ethereum smart contract.
