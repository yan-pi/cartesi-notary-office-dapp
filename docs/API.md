# Cartesi Notary DApp - API Specification

## Overview

The Cartesi Notary DApp provides two main operations:
1. **Notarize** - Submit a document for notarization (state-changing, uses `advance_state`)
2. **Verify** - Check if a document exists in the notary database (read-only, uses `inspect_state`)

All payloads are **hex-encoded** when sent to the Cartesi rollup server.

## Prerequisites for Local Testing

Before running the examples in this document, you need:

1. **Start the Cartesi node** in a separate terminal:
   ```bash
   cartesi run
   ```
   Wait until you see:
   ```
   Anvil running at http://localhost:8545
   GraphQL running at http://localhost:8080/graphql
   Inspect running at http://localhost:8080/inspect/
   ```

2. **Local Development Flags:** All `cartesi send` commands require these flags when running locally:
   - `--rpc-url http://localhost:8545` - Points to your local Anvil instance
   - `--mnemonic-index 0` - Uses the first test account from Anvil's default mnemonic

   **Without these flags**, the CLI will prompt you to select a blockchain network instead of using localhost.

3. **Two Terminal Workflow:**
   - **Terminal 1:** Run `cartesi run` (leave it running)
   - **Terminal 2:** Execute `cartesi send` commands with local flags

## Table of Contents

- [Data Types](#data-types)
- [Notarize Document](#notarize-document)
- [Verify Document](#verify-document)
- [Error Handling](#error-handling)
- [Examples](#examples)

---

## Data Types

### Document

```rust
{
  "id": String,              // UUID v4
  "content_hash": String,    // SHA-256 hash (64 hex characters)
  "file_name": String,       // Original filename
  "mime_type": String,       // MIME type (e.g., "application/pdf")
  "submitted_by": String,    // Ethereum address of submitter
  "created_at": i64          // Unix timestamp
}
```

### NotarizationReceipt

```rust
{
  "document_id": String,     // UUID matching the document
  "content_hash": String,    // SHA-256 hash
  "notarized_at": i64,       // Unix timestamp
  "block_number": u64,       // Block number at notarization time
  "proof": String            // Format: "sha256:{hash}@{timestamp}"
}
```

---

## Notarize Document

Submit a document for notarization and receive a cryptographic receipt.

### Request Type

**Endpoint:** Cartesi rollup `advance_state`

**Metadata Required:**
- `msg_sender` - Ethereum address of the sender
- `block_number` - Current block number

### Input Payload

```json
{
  "action": "notarize",
  "data": {
    "content": "<base64-encoded-document>",
    "file_name": "<filename>",
    "mime_type": "<mime-type>"
  }
}
```

**Fields:**
- `content` (String, required) - Document content encoded in base64
- `file_name` (String, required) - Original filename (max 255 chars recommended)
- `mime_type` (String, required) - MIME type (e.g., `text/plain`, `application/pdf`, `image/png`)

### Output (Notice)

**On Success:**

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

**Status:** `accept`

### Error Cases

| Error | Report Content | Status |
|-------|---------------|--------|
| Empty content | `{"error":"Empty content not allowed"}` | `reject` |
| Empty filename | `{"error":"Empty file_name not allowed"}` | `reject` |
| Duplicate document | `{"error":"Document with this content hash already exists"}` | `reject` |
| Invalid JSON | `{"error":"Invalid input format: <details>"}` | `reject` |
| Invalid base64 | `{"error":"Invalid base64 content: <details>"}` | `reject` |

### Example cURL (via Cartesi CLI)

```bash
# 1. Encode document content to base64
echo "Hello, Cartesi Notary!" | base64
# Output: SGVsbG8sIENhcnRlc2kgTm90YXJ5IQo=

# 2. Send notarization request (local development)
cartesi send generic \
  --rpc-url http://localhost:8545 \
  --mnemonic-index 0 \
  --input '{
    "action":"notarize",
    "data":{
      "content":"SGVsbG8sIENhcnRlc2kgTm90YXJ5IQo=",
      "file_name":"greeting.txt",
      "mime_type":"text/plain"
    }
  }'

# 3. Check notices for receipt
cartesi notices --rpc-url http://localhost:8545
```

### Validation Rules

1. **Content:**
   - Must be valid base64-encoded data
   - Cannot be empty after decoding
   - No size limit enforced (SQLite BLOB supports up to 2GB)

2. **File Name:**
   - Cannot be empty string
   - No path traversal validation (future enhancement)

3. **MIME Type:**
   - Cannot be empty string
   - No strict validation (accepts any string)

4. **Duplicate Detection:**
   - SHA-256 hash is calculated from decoded content
   - Database enforces UNIQUE constraint on `content_hash`
   - Same content from different users = duplicate (rejected)

---

## Verify Document

Check if a document with a given content hash has been notarized.

### Request Type

**Endpoint:** Cartesi rollup `inspect_state`

**Note:** This is a read-only operation. No state changes occur.

### Input Payload

```json
{
  "content_hash": "<64-character-hex-hash>"
}
```

**Fields:**
- `content_hash` (String, required) - SHA-256 hash in hexadecimal format (64 characters)

**Important:** Unlike `notarize`, the verify request does NOT use the `{"action":"verify","data":{...}}` wrapper when sent via `inspect_state`. The payload is just the plain `VerifyRequest` format shown above.

### Output (Report)

**Document Found:**

```json
{
  "exists": true,
  "document": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "content_hash": "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e",
    "file_name": "greeting.txt",
    "mime_type": "text/plain",
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

**Document Not Found:**

```json
{
  "exists": false,
  "document": null,
  "receipt": null
}
```

**Status:** Always `accept` (inspect operations don't reject)

### Error Cases

| Error | Report Content | Status |
|-------|---------------|--------|
| Invalid hash format | `{"error":"Invalid hash format: expected 64 hex characters"}` | `accept` |
| Invalid JSON | `{"error":"Invalid request format: <details>"}` | `accept` |

**Note:** Inspect operations always return `accept` status, even on errors. Errors are communicated via the report content.

### Example cURL (via Cartesi CLI)

```bash
# Calculate SHA-256 hash of your document
echo -n "Hello, Cartesi Notary!" | sha256sum
# Output: 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08

# Send verification request (local development)
cartesi send inspect \
  --rpc-url http://localhost:8545 \
  --payload '{
    "content_hash":"9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
  }'

# View reports
cartesi reports --rpc-url http://localhost:8545
```

### Validation Rules

1. **Content Hash:**
   - Must be exactly 64 characters
   - Must contain only hexadecimal characters (0-9, a-f, A-F)
   - Case-insensitive (internally converted to lowercase)

2. **Database Lookup:**
   - Query by hash is indexed for fast retrieval
   - Returns full document and receipt if found

---

## Error Handling

### Error Response Format

All errors are returned as JSON reports with the following structure:

```json
{
  "error": "<error message>"
}
```

### HTTP Status Codes

The DApp uses Cartesi's finish endpoint status:
- `"accept"` - Operation succeeded or inspect request completed
- `"reject"` - Operation failed (advance_state only)

### Common Error Messages

| Message | Cause | Resolution |
|---------|-------|------------|
| `"Empty content not allowed"` | Content is empty after base64 decode | Provide non-empty document |
| `"Empty file_name not allowed"` | file_name is empty string | Provide valid filename |
| `"Document with this content hash already exists"` | Duplicate notarization attempt | Document already notarized |
| `"Invalid input format: ..."` | JSON parsing failed | Check JSON syntax |
| `"Invalid base64 content: ..."` | Base64 decoding failed | Verify base64 encoding |
| `"Invalid hash format: expected 64 hex characters"` | Hash is wrong length or invalid chars | Use SHA-256 hex output |

---

## Examples

### Example 1: Notarize a Text File

```bash
# Prepare document
echo "This is my important contract" > contract.txt

# Encode to base64
CONTENT=$(cat contract.txt | base64)

# Submit notarization (local development)
cartesi send generic \
  --rpc-url http://localhost:8545 \
  --mnemonic-index 0 \
  --input "{
    \"action\":\"notarize\",
    \"data\":{
      \"content\":\"$CONTENT\",
      \"file_name\":\"contract.txt\",
      \"mime_type\":\"text/plain\"
    }
  }"

# Wait a moment, then check notices
sleep 2
cartesi notices --rpc-url http://localhost:8545
```

**Expected Output:**
```json
{
  "type": "notarization_receipt",
  "receipt": {
    "document_id": "<uuid>",
    "content_hash": "<sha256-hash>",
    "notarized_at": 1735862400,
    "block_number": 12345,
    "proof": "sha256:<hash>@<timestamp>"
  }
}
```

### Example 2: Verify Document Exists

```bash
# Get hash from previous notarization notice
HASH="a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"

# Verify document (local development)
cartesi send inspect \
  --rpc-url http://localhost:8545 \
  --payload "{
    \"content_hash\":\"$HASH\"
  }"

# Check reports
cartesi reports --rpc-url http://localhost:8545
```

**Expected Output:**
```json
{
  "exists": true,
  "document": { /* full document details */ },
  "receipt": { /* full receipt details */ }
}
```

### Example 3: Attempt Duplicate Notarization

```bash
# Try to notarize the same content again (local development)
cartesi send generic \
  --rpc-url http://localhost:8545 \
  --mnemonic-index 0 \
  --input "{
    \"action\":\"notarize\",
    \"data\":{
      \"content\":\"$CONTENT\",
      \"file_name\":\"contract-copy.txt\",
      \"mime_type\":\"text/plain\"
    }
  }"

# Check reports for error
cartesi reports --rpc-url http://localhost:8545
```

**Expected Output:**
```json
{
  "error": "Document with this content hash already exists"
}
```

**Status:** `reject`

### Example 4: Verify Non-Existent Document

```bash
# Try to verify a hash that was never notarized (local development)
cartesi send inspect \
  --rpc-url http://localhost:8545 \
  --payload '{
    "content_hash":"0000000000000000000000000000000000000000000000000000000000000000"
  }'

cartesi reports --rpc-url http://localhost:8545
```

**Expected Output:**
```json
{
  "exists": false,
  "document": null,
  "receipt": null
}
```

---

## Payload Encoding

### Hex Encoding for Cartesi Rollups

When interacting with the Cartesi rollup HTTP server directly (not via CLI), payloads must be hex-encoded:

```javascript
// JavaScript example
const payload = {
  action: "notarize",
  data: {
    content: "SGVsbG8gV29ybGQ=",
    file_name: "hello.txt",
    mime_type: "text/plain"
  }
};

const payloadJson = JSON.stringify(payload);
const payloadHex = Buffer.from(payloadJson).toString('hex');

// Send to /finish endpoint
fetch('http://127.0.0.1:5004/finish', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    status: "accept",
    payload: payloadHex
  })
});
```

### Base64 Encoding for Document Content

Document content must be base64-encoded before submitting:

```bash
# Linux/macOS
base64 < document.pdf

# Remove newlines for JSON
base64 < document.pdf | tr -d '\n'

# Python
python3 -c "import base64; print(base64.b64encode(open('document.pdf','rb').read()).decode())"
```

---

## Rate Limits and Constraints

### Current Limitations

- **No size limits** enforced on document content (SQLite supports up to 2GB BLOBs)
- **No rate limiting** implemented
- **No authentication** beyond Ethereum address from metadata
- **No access control** (anyone can verify any document)

### Recommended Constraints (for production)

- Maximum document size: 10 MB
- Rate limit: 100 requests/minute per address
- Storage limit: 100 GB total database size

---

## Versioning

**Current Version:** 1.0.0 (MVP)

This API follows semantic versioning. Breaking changes will increment the major version.

### Future API Changes (v2.0)

- GPG signature verification endpoints
- IPFS content addressing
- Batch notarization support
- Document revocation mechanism

---

## Troubleshooting

### Issue: "Chain selection prompt appears when using `cartesi send`"

**Symptoms:**
```
? Chain (Use arrow keys)
❯ Foundry
  Arbitrum Sepolia
  Base Sepolia
  ...
```

**Cause:** Missing local development flags. The CLI defaults to asking which blockchain network to use.

**Solution:** Add `--rpc-url http://localhost:8545 --mnemonic-index 0` to your command:
```bash
# Wrong (triggers chain selection):
cartesi send generic --input '...'

# Correct (uses local Anvil):
cartesi send generic \
  --rpc-url http://localhost:8545 \
  --mnemonic-index 0 \
  --input '...'
```

---

### Issue: "Connection refused" or "Failed to connect"

**Symptoms:**
```
Error: connect ECONNREFUSED 127.0.0.1:8545
```

**Cause:** Cartesi node (`cartesi run`) is not running.

**Solution:**
1. Open a separate terminal
2. Navigate to your project directory
3. Run `cartesi run`
4. Wait for "Anvil running at http://localhost:8545"
5. Try your `cartesi send` command again in the other terminal

---

### Issue: "Notices or reports are empty"

**Cause:** Request may have been rejected, or you're checking too quickly.

**Solution:**
1. Wait 2-3 seconds after sending input
2. Check reports for errors: `cartesi reports --rpc-url http://localhost:8545`
3. Verify your input JSON is valid
4. Check that base64 encoding is correct (no extra newlines)

---

### Issue: "Invalid base64 content"

**Cause:** Base64 string contains newlines or invalid characters.

**Solution:** Remove newlines when encoding:
```bash
# Wrong (includes newlines):
CONTENT=$(cat file.txt | base64)

# Correct (removes newlines):
CONTENT=$(cat file.txt | base64 | tr -d '\n')
```

---

## Support

For issues or questions, refer to:
- [README.md](../README.md) - Project overview
- [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md) - Development roadmap
- [Cartesi Documentation](https://docs.cartesi.io/)
- [Prerequisites for Local Testing](#prerequisites-for-local-testing) - Setup guide
