# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Cartesi DApp built in Rust implementing a blockchain-based notary public service. It leverages Cartesi's optimistic rollups to perform complex off-chain computations (document hashing, signature verification, timestamping) within a RISC-V Linux VM, then produces verifiable on-chain results.

## Build and Development Commands

### Building the DApp

```bash
# Build for native target (development/testing)
cargo build

# Build for Cartesi's RISC-V target (production)
cargo build --release --target riscv64gc-unknown-linux-gnu
```

Note: The RISC-V cross-compilation target must be added first:
```bash
rustup target add riscv64gc-unknown-linux-gnu
```

### Building the Docker Image

The Dockerfile uses a multi-stage build:
```bash
docker build -t cartesi-notary-dapp .
```

The build process:
1. Builder stage: Compiles Rust code for `riscv64gc-unknown-linux-gnu` target
2. Runtime stage: Creates minimal RISC-V Linux image with Cartesi machine-emulator-tools v0.14.1

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint with Clippy
cargo clippy
```

## Architecture

### Request Handling Model

The application implements a rollup-based request/response cycle:

1. **Main Loop** (`main.rs:36-66`): Continuously polls the Cartesi Rollup HTTP server at `ROLLUP_HTTP_SERVER_URL` (default: http://127.0.0.1:5004)
2. **Finish Endpoint**: Sends completion status ("accept"/"reject") and receives next request
3. **Request Routing** (`main.rs:57-64`): Dispatches based on `request_type`:
   - `"advance_state"` → `handle_advance()` - State-changing operations
   - `"inspect_state"` → `handle_inspect()` - Read-only queries

### Handler Functions

- **`handle_advance`** (`main.rs:4-15`): Processes inputs that modify state (e.g., document submissions, notarization requests). Returns "accept" or "reject".
- **`handle_inspect`** (`main.rs:17-28`): Handles read-only queries (e.g., document verification). No state changes.

Both handlers currently extract `request["data"]["payload"]` but have placeholder logic marked with `// TODO: add application logic here`.

### Planned Architecture (from PROJECT_GUIDE.md)

The notary service will follow Clean Architecture with these layers:

```
src/
├── domain/          # Core business entities and traits
│   ├── entities.rs  # Document, Signature, NotarizationReceipt
│   └── services.rs  # NotaryService, SignatureVerifier traits
├── application/     # Use case orchestration
│   ├── usecases.rs  # NotarizeDocument, VerifyDocument
│   └── handlers.rs  # Integration with handle_advance/handle_inspect
└── infrastructure/  # External integrations
    ├── database.rs  # SQLite via rusqlite
    ├── crypto.rs    # sequoia-pgp for GPG verification
    ├── storage.rs   # IPFS integration
    └── timestamp.rs # RFC3161 timestamp authorities
```

## Key Dependencies

Current (`Cargo.toml`):
- `json` (0.12): JSON parsing for request/response payloads
- `hyper` (0.14): HTTP client for Rollup server communication
- `tokio` (1.32): Async runtime

Planned (per PROJECT_GUIDE.md):
- `rusqlite`: Document/signature persistence
- `sequoia-pgp`: OpenPGP signature verification
- `tsp-http-client`: RFC3161 timestamping
- `ring` or `ed25519-dalek`: Cryptographic operations
- `ipfs-api`: Decentralized storage
- `lopdf`: PDF manipulation
- `uuid`, `sha2`: ID generation and hashing

## Environment Variables

- `ROLLUP_HTTP_SERVER_URL`: Cartesi Rollup HTTP server endpoint (default in Dockerfile: `http://127.0.0.1:5004`)

## Cartesi-Specific Considerations

### Rollup Lifecycle
1. Application sends POST to `/finish` with status
2. Server responds with 202 (no pending request) or 200 with request data
3. Application processes request via handlers
4. Loop continues with new status

### Output Mechanisms
- **Notices**: Verifiable application outputs (e.g., notarization receipts)
- **Vouchers**: On-chain executable actions (e.g., token transfers)
- **Reports**: Non-verifiable logs (e.g., error messages)

Emit these using the Cartesi Rollup HTTP server endpoints (not yet implemented in current code).

### Docker Build Context
- Builder stage runs on host architecture (amd64/arm64)
- Runtime stage uses `linux/riscv64` platform
- Cross-compilation handled by `g++-riscv64-linux-gnu` and Rust's RISC-V target
- Final binary deployed at `/opt/cartesi/dapp/dapp`
- Entry point: `rollup-init` wrapper → `dapp` binary

## Implementation Status

**Current State**: Minimal template with:
- Request polling loop
- Basic JSON request parsing
- Handler stubs

**Next Steps** (per PROJECT_GUIDE.md):
1. Implement domain entities (Document, Signature)
2. Add SQLite persistence layer
3. Integrate cryptographic verification (GPG, hashing)
4. Implement RFC3161 timestamping
5. Add IPFS storage for document content
6. Build compliance and security layers

## Testing Considerations

When writing tests for Cartesi DApps:
- Mock the HTTP server responses for unit tests
- Test request parsing and routing logic independently
- For integration tests, use Cartesi's local development environment
- Validate JSON payload structures match Cartesi Rollup spec
- Test both success and error paths for advance/inspect handlers

## Deployment

Target network: Sepolia testnet (Ethereum)

The Dockerfile labels specify:
- Cartesi SDK version: 0.9.0
- Required RAM: 128Mi

Build for deployment:
```bash
docker buildx build --platform linux/riscv64 -t cartesi-notary .
```
