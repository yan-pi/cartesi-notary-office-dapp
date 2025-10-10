# Cartesi Notary DApp - MVP Implementation Plan

**Status:** Days 1-2 Completed ✅ | Days 3-5 Remaining
**Last Updated:** 2025-01-09
**Test Status:** 24/24 passing

## 🎯 MVP Goal

Build a hash-based document notarization service on Cartesi that:
- Accepts document submissions via Cartesi rollup inputs
- Generates SHA-256 hashes and stores in SQLite
- Returns notarization receipts via Cartesi notices
- Verifies documents via inspect queries
- Runs in RISC-V Docker environment

## ✅ Completed Work

### Day 1: Foundation Layer (15 tests ✅)

**Domain Layer:**
- `src/domain/document.rs` - Document entity with SHA-256 hashing
- `src/domain/receipt.rs` - NotarizationReceipt with proof format
- Tests: Hash determinism, UUID generation, timestamp validation

**Infrastructure Layer:**
- `src/infrastructure/database.rs` - SQLite repository with CRUD operations
- Schema: `documents` table with UNIQUE constraint on `content_hash`
- Tests: Save, find_by_hash, find_by_id, duplicate detection, count

**Key Achievements:**
- SHA-256 hashing working correctly (64 hex chars)
- UNIQUE constraint prevents duplicate hashes
- In-memory SQLite for fast testing
- Custom `DatabaseError` types

**Code Metrics:**
- Production: 315 lines
- Tests: 192 lines
- Files: 8

---

### Day 2: Business Logic Layer (9 tests ✅)

**Application Layer:**
- `src/application/notarize.rs` - NotarizeUseCase with validation
- `src/application/verify.rs` - VerifyUseCase with hash validation
- `src/application/mod.rs` - VerificationResult type

**Features Implemented:**
- **NotarizeUseCase:**
  - Validates content not empty
  - Validates filename not empty
  - Checks for duplicate hashes
  - Creates Document and saves to repository
  - Generates NotarizationReceipt with proof `sha256:{hash}@{timestamp}`

- **VerifyUseCase:**
  - Validates hash is 64 hex characters
  - Queries repository by hash
  - Returns VerificationResult (exists, document, receipt)
  - Handles not found gracefully

**Error Handling:**
- `NotarizeError`: EmptyContent, EmptyFilename, DuplicateDocument
- `VerifyError`: InvalidHashFormat, DatabaseError

**Code Metrics:**
- Production: 189 lines (use cases)
- Tests: 160 lines
- Total: 24 tests passing

---

## 📋 Remaining Work

### Day 3: Cartesi Integration (Handlers)

**Goal:** Connect use cases to Cartesi rollup request/response cycle

#### 3.1 Create Request/Response Types

**File:** `src/application/types.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct NotarizeRequest {
    pub content: String,      // base64 encoded
    pub file_name: String,
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub content_hash: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum InputAction {
    #[serde(rename = "notarize")]
    Notarize { data: NotarizeRequest },

    #[serde(rename = "verify")]
    Verify { data: VerifyRequest },
}

#[derive(Debug, Serialize)]
pub struct NoticeResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub receipt: NotarizationReceipt,
}

#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub verification: VerificationResult,
}
```

#### 3.2 Implement Notice/Report Emission

**File:** `src/infrastructure/cartesi.rs`

```rust
use hyper::{Body, Client, Method, Request};
use std::error::Error;

pub async fn send_notice(
    client: &Client<hyper::client::HttpConnector>,
    server_url: &str,
    payload: &str,
) -> Result<(), Box<dyn Error>> {
    let payload_hex = hex::encode(payload);

    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("{}/notice", server_url))
        .header("content-type", "application/json")
        .body(Body::from(json::object! {
            "payload" => payload_hex
        }.dump()))?;

    let response = client.request(request).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to send notice: {}", response.status()).into());
    }

    Ok(())
}

pub async fn send_report(
    client: &Client<hyper::client::HttpConnector>,
    server_url: &str,
    payload: &str,
) -> Result<(), Box<dyn Error>> {
    let payload_hex = hex::encode(payload);

    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("{}/report", server_url))
        .header("content-type", "application/json")
        .body(Body::from(json::object! {
            "payload" => payload_hex
        }.dump()))?;

    let response = client.request(request).await?;

    if !response.status().is_success() {
        return Err(format!("Failed to send report: {}", response.status()).into());
    }

    Ok(())
}
```

#### 3.3 Update main.rs Handlers

**File:** `src/main.rs`

Replace `handle_advance` and `handle_inspect` with:

```rust
use dapp::application::{NotarizeUseCase, VerifyUseCase, InputAction, NoticeResponse, ReportResponse};
use dapp::infrastructure::{database::SqliteRepository, cartesi::{send_notice, send_report}};
use base64;

// Global repository - initialized once at startup
lazy_static::lazy_static! {
    static ref REPOSITORY: SqliteRepository =
        SqliteRepository::new("/var/lib/notary/notary.db")
            .expect("Failed to initialize database");
}

pub async fn handle_advance(
    client: &hyper::Client<hyper::client::HttpConnector>,
    server_addr: &str,
    request: JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Received advance request");

    // Extract and decode payload
    let payload_hex = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;

    let payload_bytes = hex::decode(payload_hex)?;
    let payload_str = std::str::from_utf8(&payload_bytes)?;

    // Parse input action
    let input: InputAction = serde_json::from_str(payload_str)?;

    // Get submitter from metadata
    let submitter = request["data"]["metadata"]["msg_sender"]
        .as_str()
        .unwrap_or("unknown");

    let block_number = request["data"]["metadata"]["block_number"]
        .as_u64()
        .unwrap_or(0);

    match input {
        InputAction::Notarize { data } => {
            // Decode base64 content
            let content = base64::decode(&data.content)?;

            // Execute notarization
            let notarize_usecase = NotarizeUseCase::new(Box::new(&*REPOSITORY));
            let receipt = notarize_usecase.execute(
                &content,
                &data.file_name,
                &data.mime_type,
                submitter,
                block_number,
            )?;

            // Send notice
            let response = NoticeResponse {
                response_type: "notarization_receipt".to_string(),
                receipt,
            };
            let notice_json = serde_json::to_string(&response)?;
            send_notice(client, server_addr, &notice_json).await?;

            println!("Document notarized successfully");
            Ok("accept")
        }
        InputAction::Verify { data } => {
            // Verification via advance (could also be inspect)
            let verify_usecase = VerifyUseCase::new(Box::new(&*REPOSITORY));
            let result = verify_usecase.execute(&data.content_hash)?;

            let response = ReportResponse {
                verification: result,
            };
            let report_json = serde_json::to_string(&response)?;
            send_report(client, server_addr, &report_json).await?;

            Ok("accept")
        }
    }
}

pub async fn handle_inspect(
    client: &hyper::Client<hyper::client::HttpConnector>,
    server_addr: &str,
    request: JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Received inspect request");

    // Extract and decode payload
    let payload_hex = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;

    let payload_bytes = hex::decode(payload_hex)?;
    let payload_str = std::str::from_utf8(&payload_bytes)?;

    // Parse verify request
    let verify_req: VerifyRequest = serde_json::from_str(payload_str)?;

    // Execute verification
    let verify_usecase = VerifyUseCase::new(Box::new(&*REPOSITORY));
    let result = verify_usecase.execute(&verify_req.content_hash)?;

    // Send report
    let response = ReportResponse {
        verification: result,
    };
    let report_json = serde_json::to_string(&response)?;
    send_report(client, server_addr, &report_json).await?;

    Ok("accept")
}
```

#### 3.4 Add Dependencies

**Cargo.toml:**
```toml
[dependencies]
# ... existing dependencies ...
lazy_static = "1.4"
```

#### 3.5 Testing Strategy

**Manual Testing:**
```bash
# Run locally (override RISC-V target)
cargo build --target aarch64-apple-darwin

# Test with curl (if you have a mock server)
echo '{"action":"notarize","data":{"content":"SGVsbG8gV29ybGQ=","file_name":"test.txt","mime_type":"text/plain"}}' | \
  base64 | \
  xxd -p -c 256
```

**Success Criteria:**
- [ ] Request parsing works (InputAction deserialization)
- [ ] Notarization emits notice with receipt
- [ ] Verification emits report with result
- [ ] Errors handled gracefully (reject status)

---

### Day 4: Integration Testing

**Goal:** End-to-end tests with mock rollup server

#### 4.1 Create Mock Rollup Server

**File:** `tests/integration/mock_server.rs`

```rust
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::sync::{Arc, Mutex};

pub struct MockRollupServer {
    notices: Arc<Mutex<Vec<String>>>,
    reports: Arc<Mutex<Vec<String>>>,
}

impl MockRollupServer {
    pub fn new() -> Self {
        Self {
            notices: Arc::new(Mutex::new(Vec::new())),
            reports: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn start(self) -> String {
        let notices = self.notices.clone();
        let reports = self.reports.clone();

        let make_svc = make_service_fn(move |_conn| {
            let notices = notices.clone();
            let reports = reports.clone();

            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    handle_request(req, notices.clone(), reports.clone())
                }))
            }
        });

        let addr = ([127, 0, 0, 1], 0).into();
        let server = Server::bind(&addr).serve(make_svc);
        let url = format!("http://{}", server.local_addr());

        tokio::spawn(server);
        url
    }

    pub fn get_notices(&self) -> Vec<String> {
        self.notices.lock().unwrap().clone()
    }

    pub fn get_reports(&self) -> Vec<String> {
        self.reports.lock().unwrap().clone()
    }
}

async fn handle_request(
    req: Request<Body>,
    notices: Arc<Mutex<Vec<String>>>,
    reports: Arc<Mutex<Vec<String>>>,
) -> Result<Response<Body>, hyper::Error> {
    // Extract payload from request
    // Store in appropriate vector
    // Return success response
    Ok(Response::new(Body::from("OK")))
}
```

#### 4.2 Write Integration Tests

**File:** `tests/integration/rollup_tests.rs`

```rust
#[tokio::test]
async fn test_full_notarization_workflow() {
    let mock_server = MockRollupServer::new();
    let url = mock_server.start().await;

    // Create notarize request
    let input = json::object! {
        "action" => "notarize",
        "data" => {
            "content" => base64::encode(b"test document"),
            "file_name" => "test.pdf",
            "mime_type" => "application/pdf"
        }
    };

    let request = create_advance_request(&input);

    // Call handler
    let client = hyper::Client::new();
    let status = handle_advance(&client, &url, request).await.unwrap();

    // Verify
    assert_eq!(status, "accept");
    let notices = mock_server.get_notices();
    assert_eq!(notices.len(), 1);

    // Parse notice and verify receipt
    let notice: NoticeResponse = serde_json::from_str(&notices[0]).unwrap();
    assert_eq!(notice.response_type, "notarization_receipt");
    assert_eq!(notice.receipt.content_hash.len(), 64);
}

#[tokio::test]
async fn test_duplicate_document_rejected() {
    // Submit same document twice
    // Verify second attempt returns "reject"
}

#[tokio::test]
async fn test_verification_workflow() {
    // Notarize a document
    // Query via inspect
    // Verify report contains correct data
}
```

**Success Criteria:**
- [ ] 3 integration tests passing
- [ ] Mock server captures notices/reports
- [ ] Full request/response cycle works
- [ ] Error cases handled (reject status)

---

### Day 5: Docker & Documentation

**Goal:** Verify RISC-V build and create documentation

#### 5.1 Docker Build Verification

**Commands:**
```bash
# Build Cartesi image
cartesi build

# Expected output:
# - Compiles for riscv64gc-unknown-linux-gnu
# - Binary at /opt/cartesi/dapp/dapp
# - Image size ~40-50MB
```

**Fix if needed:**
- Ensure `.cargo/config.toml` has default target set
- Verify `Cargo.toml` has `[[bin]]` section
- Check Dockerfile uses `--target riscv64gc-unknown-linux-gnu`

#### 5.2 Create API Documentation

**File:** `docs/API.md`

```markdown
# Cartesi Notary API

## Notarize Document

**Input (Advance):**
```json
{
  "action": "notarize",
  "data": {
    "content": "SGVsbG8gV29ybGQ=",  // base64 encoded
    "file_name": "document.pdf",
    "mime_type": "application/pdf"
  }
}
```

**Output (Notice):**
```json
{
  "type": "notarization_receipt",
  "receipt": {
    "document_id": "uuid-v4",
    "content_hash": "sha256_hex_64_chars",
    "notarized_at": 1234567890,
    "block_number": 12345,
    "proof": "sha256:hash@timestamp"
  }
}
```

## Verify Document

**Input (Inspect):**
```json
{
  "action": "verify",
  "data": {
    "content_hash": "sha256_hex_64_chars"
  }
}
```

**Output (Report):**
```json
{
  "verification": {
    "exists": true,
    "document": { ... },
    "receipt": { ... }
  }
}
```
```

#### 5.3 Create Demo Script

**File:** `scripts/demo.sh`

```bash
#!/bin/bash
set -e

echo "=== Cartesi Notary Demo ==="

# Start Cartesi node (in background)
cartesi run &
CARTESI_PID=$!
sleep 5

# Submit document
echo "1. Submitting document..."
cartesi send generic \
  --input '{"action":"notarize","data":{"content":"SGVsbG8gV29ybGQ=","file_name":"hello.txt","mime_type":"text/plain"}}'

# Wait for processing
sleep 2

# Verify notices
echo "2. Checking notices..."
cartesi notices

# Verify document
echo "3. Verifying document..."
# Extract hash from notice, then query
HASH="..." # From notice
cartesi send inspect \
  --payload "{\"action\":\"verify\",\"data\":{\"content_hash\":\"$HASH\"}}"

# Cleanup
kill $CARTESI_PID
echo "Demo complete!"
```

#### 5.4 Update README

**Add to README.md:**
- Build instructions
- How to run tests locally
- How to run in Docker
- API examples
- Demo walkthrough

**Success Criteria:**
- [ ] Docker image builds successfully
- [ ] Image runs in Cartesi environment
- [ ] Documentation complete
- [ ] Demo script works

---

## 🧪 Testing Reference

### Run Tests Locally

```bash
# Override RISC-V target for native testing
cargo test --lib --target aarch64-apple-darwin  # macOS ARM
cargo test --lib --target x86_64-apple-darwin   # macOS Intel

# Run specific test suite
cargo test --test unit --target aarch64-apple-darwin -- --test-threads=1

# Run with output
cargo test -- --nocapture
```

### Run Docker Build

```bash
# Build Cartesi image
cartesi build

# Run Cartesi node locally
cartesi run

# Send test input
cartesi send generic --input '{"action":"notarize",...}'
```

---

## 📊 Success Metrics

### Overall MVP Goals
- [ ] 25+ tests passing (unit + integration)
- [ ] 90%+ code coverage
- [ ] Docker builds for riscv64
- [ ] All core features working:
  - [ ] Document notarization
  - [ ] Duplicate detection
  - [ ] Verification
  - [ ] Notice emission
  - [ ] Report emission

### Code Quality
- [ ] No compiler warnings
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` clean
- [ ] All error paths tested

---

## 🔧 Troubleshooting

### Build Issues

**Problem:** Docker build fails - binary not found
**Solution:**
1. Check `Cargo.toml` has `[[bin]]` section
2. Verify Dockerfile uses `--target riscv64gc-unknown-linux-gnu`
3. Restore `.cargo/config.toml` default target

**Problem:** Tests fail locally
**Solution:**
- Override target: `cargo test --target $(rustc -vV | grep host | awk '{print $2}')`
- For M1/M2 Mac: `cargo test --target aarch64-apple-darwin`

**Problem:** Repository trait object issues
**Solution:**
- Use `Box<dyn DocumentRepository>`
- For static lifetime, use `lazy_static!` with persistent DB

### Runtime Issues

**Problem:** Database file not found in Docker
**Solution:**
- Create directory: `/var/lib/notary/`
- Initialize DB in main before handlers

**Problem:** Notice/Report not appearing
**Solution:**
- Verify hex encoding of payload
- Check HTTP status codes from rollup server
- Add debug logging

---

## 📁 Project Structure

```
cartesi-notary/
├── src/
│   ├── main.rs              # Rollup loop + handlers
│   ├── lib.rs               # Module exports
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── document.rs      # ✅ Document entity
│   │   └── receipt.rs       # ✅ NotarizationReceipt
│   ├── application/
│   │   ├── mod.rs
│   │   ├── notarize.rs      # ✅ NotarizeUseCase
│   │   ├── verify.rs        # ✅ VerifyUseCase
│   │   └── types.rs         # 🔲 Request/Response types
│   └── infrastructure/
│       ├── mod.rs
│       ├── database.rs      # ✅ SQLite repository
│       └── cartesi.rs       # 🔲 Notice/report helpers
├── tests/
│   ├── unit/
│   │   ├── mod.rs
│   │   ├── domain_tests.rs      # ✅ 8 tests
│   │   ├── database_tests.rs    # ✅ 7 tests
│   │   └── usecase_tests.rs     # ✅ 9 tests
│   └── integration/
│       ├── mod.rs               # 🔲 Integration setup
│       ├── mock_server.rs       # 🔲 Mock rollup server
│       └── rollup_tests.rs      # 🔲 E2E tests
├── docs/
│   ├── API.md               # 🔲 API documentation
│   └── DEMO.md              # 🔲 Demo walkthrough
├── scripts/
│   └── demo.sh              # 🔲 Demo script
├── Cargo.toml               # ✅ Dependencies configured
├── Dockerfile               # ✅ Multi-stage RISC-V build
├── .cargo/config.toml       # ✅ Default target
├── PROJECT_GUIDE.md         # Original comprehensive guide
├── CLAUDE.md                # Project instructions
└── IMPLEMENTATION_PLAN.md   # This file

Legend: ✅ Complete | 🔲 Remaining
```

---

## 🎓 Key Learnings

### Cartesi Patterns
1. **Rollup Loop:** POST to `/finish` → Get request → Handle → Repeat
2. **Notice vs Report:** Notices are verifiable (on-chain), reports are not
3. **Payload Encoding:** Always hex encode JSON payloads
4. **Request Types:** `advance_state` (state change), `inspect_state` (query)

### Rust Patterns
1. **Error Handling:** Custom error types with `thiserror`
2. **Repository Pattern:** `Box<dyn Trait>` for dependency injection
3. **Testing:** In-memory SQLite for fast tests
4. **Validation:** Fail fast with descriptive errors

### TDD Workflow
1. Write failing test (RED)
2. Implement minimal code (GREEN)
3. Refactor for quality (REFACTOR)
4. Repeat

---

## 📚 References

- [Cartesi Rollups Docs](https://docs.cartesi.io/cartesi-rollups/1.5/)
- [Cartesi HTTP API](https://docs.cartesi.io/cartesi-rollups/1.5/rollups-apis/)
- [Cartesi Examples](https://github.com/cartesi/rollups-examples)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Hyper HTTP Client](https://docs.rs/hyper/latest/hyper/)

---

**Next Steps:** Proceed to Day 3 - Implement Cartesi integration with handlers and notice/report emission.
