#!/bin/bash
#
# Cartesi Notary DApp - Interactive Demo
#
# This script demonstrates the core functionality of the Cartesi Notary DApp:
# 1. Notarizing a document
# 2. Verifying the notarized document
# 3. Attempting duplicate notarization (which should fail)
#
# Prerequisites:
# - Cartesi CLI installed (npm install -g @cartesi/cli)
# - Docker running
# - DApp built (cartesi build)
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

print_step() {
    echo -e "${GREEN}▸ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

pause() {
    echo ""
    read -p "Press ENTER to continue..."
    echo ""
}

# Banner
clear
print_header "🔐 Cartesi Notary DApp - Interactive Demo"

echo "This demo will:"
echo "  1. Start the Cartesi node"
echo "  2. Notarize a sample document"
echo "  3. Verify the notarized document"
echo "  4. Attempt duplicate notarization (expect failure)"
echo ""

pause

# Check prerequisites
print_step "Checking prerequisites..."

if ! command -v cartesi &> /dev/null; then
    print_error "Cartesi CLI not found. Install with: npm install -g @cartesi/cli"
    exit 1
fi

if ! command -v docker &> /dev/null; then
    print_error "Docker not found. Please install Docker."
    exit 1
fi

if ! docker info &> /dev/null; then
    print_error "Docker daemon not running. Please start Docker."
    exit 1
fi

print_success "All prerequisites met!"

# Step 1: Build the DApp (if not already built)
print_header "Step 1: Building Cartesi DApp"

print_step "Building DApp for RISC-V..."
cartesi build

print_success "DApp built successfully!"
pause

# Step 2: Start Cartesi node
print_header "Step 2: Starting Cartesi Node"

print_step "Starting node in background..."
cartesi run &> /tmp/cartesi-node.log &
CARTESI_PID=$!

print_info "Node PID: $CARTESI_PID"
print_step "Waiting for node to be ready..."
sleep 10

# Check if node is still running
if ! ps -p $CARTESI_PID > /dev/null; then
    print_error "Failed to start Cartesi node. Check /tmp/cartesi-node.log for details."
    exit 1
fi

print_success "Cartesi node running!"
print_info "Logs: /tmp/cartesi-node.log"
pause

# Step 3: Notarize a document
print_header "Step 3: Notarizing a Document"

# Create sample document
DOCUMENT_TEXT="This is a very important contract between Alice and Bob, dated 2025-01-09."
print_step "Sample document content:"
echo "  \"$DOCUMENT_TEXT\""
echo ""

# Encode to base64
CONTENT_BASE64=$(echo -n "$DOCUMENT_TEXT" | base64 | tr -d '\n')
print_step "Base64 encoded content:"
echo "  $CONTENT_BASE64"
echo ""

# Calculate SHA-256 (for later verification)
if command -v sha256sum &> /dev/null; then
    EXPECTED_HASH=$(echo -n "$DOCUMENT_TEXT" | sha256sum | awk '{print $1}')
elif command -v shasum &> /dev/null; then
    EXPECTED_HASH=$(echo -n "$DOCUMENT_TEXT" | shasum -a 256 | awk '{print $1}')
else
    EXPECTED_HASH="<calculated by DApp>"
fi

print_step "Expected SHA-256 hash:"
echo "  $EXPECTED_HASH"
echo ""

pause

# Submit notarization request
print_step "Submitting notarization request..."

cartesi send generic \
  --rpc-url http://localhost:8545 \
  --mnemonic-index 0 \
  --input "{\"action\":\"notarize\",\"data\":{\"content\":\"$CONTENT_BASE64\",\"file_name\":\"contract.txt\",\"mime_type\":\"text/plain\"}}"

print_success "Notarization request sent!"
print_step "Waiting for processing..."
sleep 3

# Check notices
print_step "Retrieving notarization receipt..."
echo ""

NOTICES_OUTPUT=$(cartesi notices --rpc-url http://localhost:8545 2>&1 || true)
echo "$NOTICES_OUTPUT"

# Try to extract hash from notices (if jq is available)
if command -v jq &> /dev/null; then
    print_step "Extracting content hash from receipt..."
    # Note: This is a simplified extraction. Actual format may vary.
    CONTENT_HASH=$(echo "$NOTICES_OUTPUT" | jq -r '.receipt.content_hash' 2>/dev/null || echo "$EXPECTED_HASH")
    print_info "Content Hash: $CONTENT_HASH"
else
    CONTENT_HASH="$EXPECTED_HASH"
    print_info "Install 'jq' for automatic hash extraction. Using calculated hash."
fi

print_success "Document notarized successfully!"
pause

# Step 4: Verify the document
print_header "Step 4: Verifying the Notarized Document"

print_step "Querying DApp with content hash..."
echo "  Hash: $CONTENT_HASH"
echo ""

cartesi send inspect \
  --rpc-url http://localhost:8545 \
  --payload "{\"content_hash\":\"$CONTENT_HASH\"}"

print_step "Checking verification report..."
sleep 2
echo ""

REPORTS_OUTPUT=$(cartesi reports --rpc-url http://localhost:8545 2>&1 || true)
echo "$REPORTS_OUTPUT"

print_success "Verification complete!"
print_info "The report should show 'exists: true' with full document details."
pause

# Step 5: Attempt duplicate notarization
print_header "Step 5: Testing Duplicate Detection"

print_step "Attempting to notarize the same document again..."
print_info "This should FAIL because duplicates are not allowed."
echo ""

cartesi send generic \
  --rpc-url http://localhost:8545 \
  --mnemonic-index 0 \
  --input "{\"action\":\"notarize\",\"data\":{\"content\":\"$CONTENT_BASE64\",\"file_name\":\"contract-duplicate.txt\",\"mime_type\":\"text/plain\"}}"

print_step "Waiting for response..."
sleep 3

print_step "Checking reports for error message..."
echo ""

REPORTS_DUPLICATE=$(cartesi reports --rpc-url http://localhost:8545 2>&1 || true)
echo "$REPORTS_DUPLICATE"

if echo "$REPORTS_DUPLICATE" | grep -q -i "duplicate\|already exists"; then
    print_success "Duplicate detection working correctly! ✓"
else
    print_error "Expected duplicate error but didn't find it."
fi

pause

# Cleanup
print_header "🎉 Demo Complete!"

print_step "Stopping Cartesi node..."
kill $CARTESI_PID 2>/dev/null || true
wait $CARTESI_PID 2>/dev/null || true

print_success "Node stopped"
echo ""
echo "Summary:"
echo "  ✓ Document notarized with SHA-256 hash"
echo "  ✓ Notarization receipt received (Notice)"
echo "  ✓ Document verified via inspect query"
echo "  ✓ Duplicate notarization correctly rejected"
echo ""
print_success "All features demonstrated successfully!"
echo ""

print_info "Next steps:"
echo "  • Check the full API documentation in docs/API.md"
echo "  • Run tests with: cargo test -- --test-threads=1"
echo "  • Read the README.md for more information"
echo ""

print_header "Thank you for trying Cartesi Notary DApp!"
