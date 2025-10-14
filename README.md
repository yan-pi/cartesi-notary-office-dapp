# Cartesi Notary DApp

A blockchain-based document notarization service built on Cartesi's optimistic rollups. This DApp leverages RISC-V computation to provide tamper-proof, verifiable document notarization with SHA-256 hashing and timestamp proofs.

## Overview

The Cartesi Notary DApp allows users to:
- **Notarize documents** - Submit documents and receive cryptographic receipts
- **Verify documents** - Check if a document has been notarized by content hash
- **Detect duplicates** - Prevent double-notarization of the same content
- **Generate proofs** - Receive verifiable proof of notarization with block number and timestamp

All computations run off-chain in a RISC-V Linux VM (Cartesi Machine), providing Ethereum-level security with minimal gas costs.

## Architecture

Built following **Clean Architecture** principles:

```
┌─────────────────────────────────────────────────┐
│                  Handlers Layer                 │
│   (Cartesi rollup request/response processing)  │
└─────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│               Application Layer                 │
│   (Business logic: NotarizeUseCase, Verify)     │
└─────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│                 Domain Layer                    │
│     (Core entities: Document, Receipt)          │
└─────────────────────────────────────────────────┘
                       ↓
┌─────────────────────────────────────────────────┐
│            Infrastructure Layer                 │
│   (SQLite database, Cartesi HTTP integration)   │
└─────────────────────────────────────────────────┘
```

### Tech Stack

- **Language:** Rust (stable)
- **Database:** SQLite with rusqlite (bundled for RISC-V)
- **Hashing:** SHA-256 via sha2 crate
- **HTTP:** Hyper async client
- **Serialization:** serde + serde_json
- **Testing:** 44 tests (24 unit + 20 integration)

## Prerequisites

- **Rust** 1.70+ with `riscv64gc-unknown-linux-gnu` target
- **Docker** for Cartesi builds
- **Cartesi CLI** (`npm install -g @cartesi/cli`)

### Install RISC-V Target

```bash
rustup target add riscv64gc-unknown-linux-gnu
```

## Quick Start

### 1. Build Natively (for testing)

```bash
# Build for your host platform
cargo build --release

# Run unit tests
cargo test --lib

# Run all tests (requires serial execution for integration tests)
cargo test -- --test-threads=1
```

### 2. Build for Cartesi (RISC-V)

```bash
# Build Docker image with RISC-V binary
cartesi build

# Terminal 1: Run Cartesi node locally
cartesi run

# Terminal 2: Send a test input (after node starts)
# Wait for "Anvil running at http://localhost:8545" message first!
cartesi send generic \
  --rpc-url http://localhost:8545 \
  --mnemonic-index 0 \
  --input '{"action":"notarize","data":{"content":"SGVsbG8gV29ybGQ=","file_name":"hello.txt","mime_type":"text/plain"}}'

# Check the notarization receipt
cartesi notices --rpc-url http://localhost:8545
```

**Important:**
- You MUST include `--rpc-url http://localhost:8545 --mnemonic-index 0` for local development
- Without these flags, you'll get a chain selection prompt instead
- See [docs/API.md](docs/API.md) for more examples and troubleshooting

## Testing

### Run All Tests

```bash
# On macOS ARM
cargo test --target aarch64-apple-darwin -- --test-threads=1

# On macOS Intel
cargo test --target x86_64-apple-darwin -- --test-threads=1

# On Linux
cargo test --target x86_64-unknown-linux-gnu -- --test-threads=1
```

**Important:** Integration tests require `--test-threads=1` due to shared environment variable usage (`NOTARY_DB_PATH`).

### Run Specific Test Suites

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration -- --test-threads=1

# With output
cargo test -- --nocapture --test-threads=1
```

### Test Coverage

- **44 tests total**: 24 unit + 20 integration
- **Unit tests**: Domain entities, database layer, use cases
- **Integration tests**: End-to-end workflows with mock Cartesi server

## API Usage

See [docs/API.md](docs/API.md) for detailed API specification.

### Notarize a Document

**Request (advance_state input):**
```json
{
  "action": "notarize",
  "data": {
    "content": "SGVsbG8gV29ybGQ=",  // base64-encoded document
    "file_name": "document.pdf",
    "mime_type": "application/pdf"
  }
}
```

**Response (Notice):**
```json
{
  "type": "notarization_receipt",
  "receipt": {
    "document_id": "550e8400-e29b-41d4-a716-446655440000",
    "content_hash": "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e",
    "notarized_at": 1735862400,
    "block_number": 12345,
    "proof": "sha256:a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e@1735862400"
  }
}
```

### Verify a Document

**Request (inspect_state input):**
```json
{
  "content_hash": "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
}
```

**Response (Report):**
```json
{
  "exists": true,
  "document": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "content_hash": "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e",
    "file_name": "document.pdf",
    "mime_type": "application/pdf",
    "submitted_by": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "created_at": 1735862400
  },
  "receipt": {
    "document_id": "550e8400-e29b-41d4-a716-446655440000",
    "content_hash": "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e",
    "notarized_at": 1735862400,
    "block_number": 12345,
    "proof": "sha256:a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e@1735862400"
  }
}
```

## Project Structure

```
final-project/
├── src/
│   ├── main.rs                    # Entry point, rollup loop
│   ├── lib.rs                     # Public module exports
│   ├── handlers.rs                # Advance/inspect handlers
│   ├── domain/
│   │   ├── mod.rs                 # Domain exports
│   │   ├── document.rs            # Document entity with SHA-256
│   │   └── receipt.rs             # NotarizationReceipt
│   ├── application/
│   │   ├── mod.rs                 # Application exports
│   │   ├── notarize.rs            # NotarizeUseCase
│   │   ├── verify.rs              # VerifyUseCase
│   │   └── types.rs               # Request/Response types
│   └── infrastructure/
│       ├── mod.rs                 # Infrastructure exports
│       ├── database.rs            # SQLite repository
│       └── cartesi.rs             # Notice/Report emission
├── tests/
│   ├── unit/                      # Unit tests
│   │   ├── mod.rs
│   │   ├── database_tests.rs
│   │   ├── document_tests.rs
│   │   ├── notarize_tests.rs
│   │   └── verify_tests.rs
│   └── integration/               # Integration tests
│       ├── mod.rs
│       ├── mock_server.rs         # Mock Cartesi HTTP server
│       ├── helpers.rs             # Test utilities
│       └── rollup_tests.rs        # End-to-end tests
├── docs/
│   └── API.md                     # Detailed API documentation
├── scripts/
│   └── demo.sh                    # Interactive demo script
├── Cargo.toml                     # Dependencies and config
├── Dockerfile                     # Multi-stage RISC-V build
├── CLAUDE.md                      # Claude Code instructions
├── PROJECT_GUIDE.md               # Original project specification
├── IMPLEMENTATION_PLAN.md         # Implementation roadmap
└── README.md                      # This file
```

## Configuration

### Environment Variables

- `ROLLUP_HTTP_SERVER_URL` - Cartesi rollup HTTP server endpoint (default: `http://127.0.0.1:5004`)
- `NOTARY_DB_PATH` - Database file path (default: `/var/lib/notary/notary.db`, falls back to in-memory)

### Database

The DApp uses SQLite with the following schema:

```sql
CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    content_hash TEXT UNIQUE NOT NULL,
    file_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    submitted_by TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_content_hash ON documents(content_hash);
CREATE INDEX idx_created_at ON documents(created_at);
```

**Duplicate Prevention:** The `UNIQUE` constraint on `content_hash` ensures no document can be notarized twice.

## Key Features

### Implemented

- [x] SHA-256 document hashing
- [x] SQLite persistence with UNIQUE constraints
- [x] Duplicate detection
- [x] Document verification by hash
- [x] Notarization receipts with proofs
- [x] Cartesi notice emission (verifiable on-chain)
- [x] Cartesi report emission (query results)
- [x] Error handling (invalid JSON, invalid base64, etc.)
- [x] 44 comprehensive tests
- [x] RISC-V Docker build

### Future Enhancements

- [ ] GPG signature verification
- [ ] IPFS integration for document storage
- [ ] RFC3161 timestamp authority integration
- [ ] Multi-signature support
- [ ] PDF metadata extraction
- [ ] On-chain vouchers for certificate issuance

## Success Metrics

- ✅ **44/25+ tests passing** (176% of target)
- ✅ **Docker builds for riscv64**
- ✅ **No compiler warnings**
- ✅ **Code formatted with rustfmt**
- ✅ **Clippy clean**
- ✅ **All core features working**

## 🤝 Contributing

This is a university project (UFBA final project). For questions or suggestions, please refer to the project documentation.

## References

- [Cartesi Documentation](https://docs.cartesi.io/)
- [Cartesi Rollups](https://docs.cartesi.io/cartesi-rollups/)
- [Rust SQLite](https://docs.rs/rusqlite/)
- [PROJECT_GUIDE.md](PROJECT_GUIDE.md) - Original specification
- [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) - Development roadmap
- [docs/API.md](docs/API.md) - API specification

**MVP Complete** - Ready for deployment and testing on Cartesi network.

Days 1-4 implemented following TDD approach with comprehensive test coverage and documentation.
