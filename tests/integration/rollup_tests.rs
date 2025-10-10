use super::helpers::*;
use super::mock_server::MockRollupServer;
use dapp::handlers::{handle_advance, handle_inspect};

#[tokio::test]
async fn test_notarize_document_workflow() {
    // Start mock server
    let server = MockRollupServer::new();
    let server_url = server.start().await;

    // Wait for server to be ready
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create notarize request
    let content = b"Hello, Cartesi Notary!";
    let payload = create_notarize_payload(content, "test.txt", "text/plain");
    let request = create_advance_request(&payload, "0x1234567890abcdef", 100);

    // Create HTTP client
    let client = hyper::Client::new();

    // Call handler
    let result = handle_advance(&client, &server_url, request).await;

    // Should succeed
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "accept");

    // Wait for async processing
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify notice was sent
    let notices = server.get_notices();
    assert_eq!(notices.len(), 1, "Should have exactly one notice");

    // Parse notice
    let notice_json: serde_json::Value = serde_json::from_str(&notices[0]).unwrap();
    assert_eq!(notice_json["type"], "notarization_receipt");

    let receipt = &notice_json["receipt"];
    assert!(!receipt["document_id"].as_str().unwrap().is_empty());
    assert_eq!(receipt["content_hash"].as_str().unwrap().len(), 64); // SHA-256
    assert_eq!(receipt["block_number"], 100);
    assert!(receipt["proof"].as_str().unwrap().starts_with("sha256:"));
}

#[tokio::test]
async fn test_notarize_duplicate_rejected() {
    let _db = TestDatabase::new(); // Set up persistent database for this test
    let server = MockRollupServer::new();
    let server_url = server.start().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = hyper::Client::new();
    let content = b"Same content";
    let payload = create_notarize_payload(content, "file1.txt", "text/plain");

    // First notarization
    let request1 = create_advance_request(&payload, "0x111", 100);
    let result1 = handle_advance(&client, &server_url, request1).await;
    assert_eq!(result1.unwrap(), "accept");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    server.clear(); // Clear first notice

    // Second notarization with same content
    let request2 = create_advance_request(&payload, "0x222", 101);
    let result2 = handle_advance(&client, &server_url, request2).await;

    // Should be rejected due to duplicate
    assert_eq!(result2.unwrap(), "reject");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Should have error report
    let reports = server.get_reports();
    assert!(!reports.is_empty());
    assert!(reports[0].contains("error") || reports[0].contains("Duplicate"));
}

#[tokio::test]
async fn test_verify_existing_document() {
    let _db = TestDatabase::new(); // Set up persistent database for this test
    let server = MockRollupServer::new();
    let server_url = server.start().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = hyper::Client::new();
    let content = b"Content to verify";

    // First, notarize a document
    let notarize_payload = create_notarize_payload(content, "doc.txt", "text/plain");
    let notarize_req = create_advance_request(&notarize_payload, "0x123", 100);
    handle_advance(&client, &server_url, notarize_req)
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Get the hash from the notice
    let notices = server.get_notices();
    let notice_json: serde_json::Value = serde_json::from_str(&notices[0]).unwrap();
    let content_hash = notice_json["receipt"]["content_hash"]
        .as_str()
        .unwrap()
        .to_string();

    server.clear();

    // Now verify it via inspect
    let verify_payload = create_verify_payload(&content_hash);
    let verify_req = create_inspect_request(&verify_payload);
    let result = handle_inspect(&client, &server_url, verify_req).await;

    assert_eq!(result.unwrap(), "accept");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check report
    let reports = server.get_reports();
    assert_eq!(reports.len(), 1);

    let report_json: serde_json::Value = serde_json::from_str(&reports[0]).unwrap();
    assert_eq!(report_json["exists"], true);
    assert!(report_json["document"].is_object());
    assert!(report_json["receipt"].is_object());
}

#[tokio::test]
async fn test_verify_nonexistent_document() {
    let _db = TestDatabase::new(); // Set up persistent database for this test
    let server = MockRollupServer::new();
    let server_url = server.start().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = hyper::Client::new();

    // Try to verify a hash that doesn't exist
    let fake_hash = "a".repeat(64);
    let verify_payload = create_verify_payload(&fake_hash);
    let verify_req = create_inspect_request(&verify_payload);
    let result = handle_inspect(&client, &server_url, verify_req).await;

    assert_eq!(result.unwrap(), "accept"); // Inspect always accepts

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Check report
    let reports = server.get_reports();
    assert_eq!(reports.len(), 1);

    let report_json: serde_json::Value = serde_json::from_str(&reports[0]).unwrap();
    assert_eq!(report_json["exists"], false);
    assert!(report_json["document"].is_null());
    assert!(report_json["receipt"].is_null());
}

#[tokio::test]
async fn test_invalid_json_rejected() {
    let server = MockRollupServer::new();
    let server_url = server.start().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = hyper::Client::new();

    // Send invalid JSON
    let invalid_payload = "not valid json {{{";
    let request = create_advance_request(invalid_payload, "0x123", 100);
    let result = handle_advance(&client, &server_url, request).await;

    // Should be rejected
    assert_eq!(result.unwrap(), "reject");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Should have error report
    let reports = server.get_reports();
    assert!(!reports.is_empty());
    assert!(reports[0].contains("error"));
}

#[tokio::test]
async fn test_invalid_base64_rejected() {
    let server = MockRollupServer::new();
    let server_url = server.start().await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = hyper::Client::new();

    // Create payload with invalid base64
    let invalid_payload = r#"{"action":"notarize","data":{"content":"!!!invalid-base64!!!","file_name":"test.txt","mime_type":"text/plain"}}"#;
    let request = create_advance_request(invalid_payload, "0x123", 100);
    let result = handle_advance(&client, &server_url, request).await;

    // Should be rejected
    assert_eq!(result.unwrap(), "reject");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Should have error report
    let reports = server.get_reports();
    assert!(!reports.is_empty());
    assert!(reports[0].contains("error"));
}
